use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
};

use bitcoin::{
    absolute::LockTime,
    consensus::encode,
    hashes::Hash,
    key::TapTweak,
    psbt::Input,
    secp256k1::{Message, Secp256k1},
    sighash::{Prevouts, SighashCache},
    taproot::{LeafVersion, Signature, TaprootBuilder},
    transaction::Version,
    Address, Amount, Network, OutPoint, Psbt, ScriptBuf, Sequence, TapSighashType, Transaction,
    TxIn, TxOut, Witness,
};

use crate::{
    electrumx::{Api, ElectrumX},
    engine::tx::{Payload, PayloadWrapper},
    model::{Ft, Utxo},
    utils::{self, sequence_ranges_by_cpus},
    wallet::Wallet,
    AnyhowResult,
};

use super::{
    tx::{Fees, TransactionData},
    wallet::EngineWallet,
    OUTPUT_BYTES_BASE,
};

pub struct Miner {
    pub network: Network,
    pub electrumx: ElectrumX,
    pub wallets: Vec<EngineWallet>,
    pub ticker: String,
    pub max_fee: u64,
}

impl Miner {
    pub async fn mine(&self, wallet: &EngineWallet) -> AnyhowResult<()> {
        let tx_data = self.prepare_data(wallet).await?;

        let TransactionData {
            secp,
            satsbyte,
            bitwork_info_commit,
            additional_outputs,
            reveal_script,
            reveal_spend_info,
            fees,
            funding_utxo,
        } = tx_data;

        let reveal_spk = ScriptBuf::new_p2tr(
            &secp,
            reveal_spend_info.internal_key(),
            reveal_spend_info.merkle_root(),
        );
        let funding_spk = wallet.funding.address.script_pubkey();

        let commit_input = vec![TxIn {
            previous_output: OutPoint::new(funding_utxo.txid.parse()?, funding_utxo.vout),
            ..Default::default()
        }];
        let commit_output = assemble_commit_output(
            fees.clone(),
            reveal_spk.clone(),
            funding_utxo.clone(),
            funding_spk.clone(),
            satsbyte,
            OUTPUT_BYTES_BASE,
        );

        let commit_prevouts = vec![TxOut {
            value: Amount::from_sat(funding_utxo.value),
            script_pubkey: funding_spk.clone(),
        }];

        let mut ts: Vec<JoinHandle<AnyhowResult<()>>> = vec![];
        let solution_found = Arc::new(AtomicBool::new(false));

        let maybe_commit_tx = Arc::new(Mutex::new(None));

        sequence_ranges_by_cpus(u32::MAX)
            .into_iter()
            .enumerate()
            .for_each(|(i, r)| {
                tracing::info!("spawning thread {i} for sequence range {r:?}");

                let secp = secp.clone();
                let bitwork_info_commit = bitwork_info_commit.clone();
                let funding_kp = wallet.funding.pair.tap_tweak(&secp, None).to_inner();
                let funding_xpk = wallet.funding.x_only_public_key;
                let input = commit_input.clone();
                let output = commit_output.clone();
                let prevouts = commit_prevouts.clone();
                let hash_ty = TapSighashType::Default;
                let solution_found = solution_found.clone();
                let maybe_tx = maybe_commit_tx.clone();

                ts.push(std::thread::spawn(move || {
                    for s in r {
                        if solution_found.load(Ordering::Relaxed) {
                            return Ok(());
                        }

                        let mut psbt = Psbt::from_unsigned_tx(Transaction {
                            version: Version::ONE,
                            lock_time: LockTime::ZERO,
                            input: {
                                let mut i = input.clone();

                                i[0].sequence = Sequence(s);

                                i
                            },
                            output: output.clone(),
                        })?;
                        let tap_key_sig = {
                            let h = SighashCache::new(&psbt.unsigned_tx)
                                .taproot_key_spend_signature_hash(
                                    0,
                                    &Prevouts::All(&prevouts),
                                    hash_ty,
                                )?;
                            let m = Message::from_digest(h.to_byte_array());

                            Signature {
                                sig: secp.sign_schnorr(&m, &funding_kp),
                                hash_ty,
                            }
                        };

                        psbt.inputs[0] = Input {
                            witness_utxo: Some(prevouts[0].clone()),
                            final_script_witness: {
                                let mut w = Witness::new();

                                w.push(tap_key_sig.to_vec());

                                Some(w)
                            },
                            tap_key_sig: Some(tap_key_sig),
                            tap_internal_key: Some(funding_xpk),
                            ..Default::default()
                        };

                        tracing::trace!("{psbt:#?}");

                        let tx = psbt.extract_tx_unchecked_fee_rate();
                        let txid = tx.txid();

                        if txid
                            .to_string()
                            .trim_start_matches("0x")
                            .starts_with(&bitwork_info_commit)
                        {
                            tracing::info!("solution found");
                            tracing::info!("sequence {s}");
                            tracing::info!("commit txid {txid}");
                            tracing::info!("commit tx {tx:#?}");

                            solution_found.store(true, Ordering::Relaxed);
                            *maybe_tx.lock().unwrap() = Some(tx);

                            return Ok(());
                        }
                    }

                    Ok(())
                }));
            });

        for t in ts {
            t.join().unwrap()?;
        }

        let commit_tx = maybe_commit_tx.lock().unwrap().take().unwrap();

        self.electrumx
            .broadcast(encode::serialize_hex(&commit_tx))
            .await?;

        let commit_txid = commit_tx.txid();
        let commit_txid_ = self
            .electrumx
            .wait_until_utxo(
                Address::from_script(&reveal_spk, self.network)?.to_string(),
                fees.reveal_and_outputs,
            )
            .await?
            .txid;

        assert_eq!(commit_txid, commit_txid_.parse()?);

        let mut reveal_psbt = Psbt::from_unsigned_tx(Transaction {
            version: Version::ONE,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::new(commit_txid, 0),
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                ..Default::default()
            }],
            output: additional_outputs,
        })?;
        let reveal_st = TapSighashType::SinglePlusAnyoneCanPay;
        let reveal_tks = {
            let lh = reveal_script.tapscript_leaf_hash();
            let h = SighashCache::new(&reveal_psbt.unsigned_tx)
                .taproot_script_spend_signature_hash(
                    0,
                    &Prevouts::One(0, commit_output[0].clone()),
                    lh,
                    reveal_st,
                )?;
            let m = Message::from_digest(h.to_byte_array());

            Signature {
                sig: secp.sign_schnorr(&m, &wallet.funding.pair),
                hash_ty: reveal_st,
            }
        };

        reveal_psbt.inputs[0] = Input {
            witness_utxo: Some(commit_output[0].clone()),
            tap_internal_key: Some(reveal_spend_info.internal_key()),
            tap_merkle_root: reveal_spend_info.merkle_root(),
            final_script_witness: {
                let mut w = Witness::new();

                w.push(reveal_tks.to_vec());
                w.push(reveal_script.as_bytes());
                w.push(
                    reveal_spend_info
                        .control_block(&(reveal_script, LeafVersion::TapScript))
                        .unwrap()
                        .serialize(),
                );

                Some(w)
            },
            ..Default::default()
        };

        let reveal_tx = reveal_psbt.extract_tx_unchecked_fee_rate();
        let reveal_txid = reveal_tx.txid();

        tracing::info!("reveal txid {reveal_txid}");
        tracing::info!("reveal tx {reveal_tx:#?}");

        self.electrumx
            .broadcast(encode::serialize_hex(&reveal_tx))
            .await?;

        Ok(())
    }

    async fn prepare_data(&self, wallet: &EngineWallet) -> AnyhowResult<TransactionData> {
        let ft = self.validate().await?;

        let secp = Secp256k1::new();
        let satsbyte = if self.network == Network::Bitcoin {
            utils::query_fee().await?.min(self.max_fee) + 5
        } else {
            5
        };

        let additional_outputs = vec![TxOut {
            value: Amount::from_sat(ft.mint_amount),
            script_pubkey: wallet.stash.address.script_pubkey(),
        }];

        let payload = PayloadWrapper {
            args: {
                let (time, nonce) = utils::time_nonce();

                Payload {
                    bitworkc: ft.mint_bitworkc.clone(),
                    mint_ticker: ft.ticker.clone(),
                    nonce,
                    time,
                }
            },
        };

        let payload_encoded = utils::cbor(&payload)?;

        let reveal_script =
            utils::build_reveal_script(&wallet.funding.x_only_public_key, "dmt", &payload_encoded);

        let reveal_spend_info = TaprootBuilder::new()
            .add_leaf(0, reveal_script.clone())?
            .finalize(&secp, wallet.funding.x_only_public_key)
            .unwrap();

        let fees = super::fees_of(
            satsbyte,
            reveal_script.as_bytes().len(),
            &additional_outputs,
        );

        let funding_utxo = self
            .electrumx
            .wait_until_utxo(
                wallet.funding.address.to_string(),
                fees.commit_and_reveal_and_outputs,
            )
            .await?;

        Ok(TransactionData {
            secp,
            satsbyte,
            bitwork_info_commit: ft.mint_bitworkc,
            additional_outputs,
            reveal_script,
            reveal_spend_info,
            fees,
            funding_utxo,
        })
    }

    async fn validate(&self) -> AnyhowResult<Ft> {
        let id = self
            .electrumx
            .get_by_ticker(&self.ticker)
            .await?
            .atomical_id;
        let response = self.electrumx.get_ft_info(id).await?;
        let global = response.global.unwrap();
        let ft = response.result;

        if ft.ticker != self.ticker {
            Err(anyhow::anyhow!("ticker mismatch"))?;
        }
        if ft.subtype != "decentralized" {
            Err(anyhow::anyhow!("not decentralized"))?;
        }
        if ft.mint_height > global.height + 1 {
            Err(anyhow::anyhow!("mint height mismatch"))?;
        }
        if ft.mint_amount == 0 || ft.mint_amount >= 100_000_000 {
            Err(anyhow::anyhow!("mint amount mismatch"))?;
        }

        Ok(ft)
    }
}

pub struct MinerBuilder {
    pub network: Network,
    pub electrumx: String,
    pub wallet_dir: PathBuf,
    pub ticker: String,
    pub max_fee: u64,
}

impl MinerBuilder {
    pub fn new(
        network: Network,
        electrumx: impl Into<String>,
        wallet_dir: impl Into<String>,
        ticker: impl Into<String>,
        max_fee: u64,
    ) -> Self {
        Self {
            network,
            electrumx: electrumx.into(),
            wallet_dir: wallet_dir.into().into(),
            ticker: ticker.into(),
            max_fee,
        }
    }

    pub fn build(&self) -> AnyhowResult<Miner> {
        let electrumx = ElectrumX::new(self.network, self.electrumx.as_str(), 30)?;
        let wallets = Wallet::load_wallets(self.wallet_dir.as_path())
            .into_iter()
            .map(|rw| EngineWallet::from_raw_wallet(rw, self.network))
            .collect::<AnyhowResult<_>>()?;

        Ok(Miner {
            network: self.network,
            electrumx,
            wallets,
            ticker: self.ticker.clone(),
            max_fee: self.max_fee,
        })
    }
}

fn assemble_commit_output(
    fees: Fees,
    reveal_spk: ScriptBuf,
    funding_utxo: Utxo,
    funding_spk: ScriptBuf,
    satsbyte: u64,
    output_bytes_bass: f64,
) -> Vec<TxOut> {
    let spend = TxOut {
        value: Amount::from_sat(fees.reveal_and_outputs),
        script_pubkey: reveal_spk.clone(),
    };

    let refund = assemble_refund(
        fees,
        funding_utxo.value,
        funding_spk,
        satsbyte,
        output_bytes_bass,
    );

    match refund {
        Some(r) => vec![spend, r],
        None => vec![spend],
    }
}

fn assemble_refund(
    fees: Fees,
    funding_value: u64,
    funding_spk: ScriptBuf,
    satsbyte: u64,
    output_bytes_bass: f64,
) -> Option<TxOut> {
    let r = funding_value
        .saturating_sub(fees.reveal_and_outputs)
        .saturating_sub(fees.commit + (output_bytes_bass * satsbyte as f64).floor() as u64);

    if r > 0 {
        Some(TxOut {
            value: Amount::from_sat(r),
            script_pubkey: funding_spk.clone(),
        })
    } else {
        None
    }
}
