use std::{collections::HashMap, fs, path::Path, sync::mpsc};

use colored::Colorize;

use crate::{
    block::{Block, ProposedBlock},
    crypto::{self, key::PublicKey, KeyPair},
    settings::Settings,
    storage::{self, Store},
    tx::{self, create_signed, SignedTransaction},
};

// TODO: Difficulty adjustment
pub static DIFFICULTY: usize = 2;
pub static GENESIS_PREV_BLOCK_HASH: &str =
    "000000000000000000000000000000000000000000000000000000000000000";

pub struct Node {
    pub mempool: HashMap<String, SignedTransaction>,
    pub keypair: KeyPair,
    pub db_blocks: Store,
    pub db_blocks_metadata: Store,
    pub db_balances: Store,

    block_tx: mpsc::Sender<Block>,
    transaction_tx: mpsc::Sender<SignedTransaction>,
}

impl Node {
    pub fn new(
        block_tx: mpsc::Sender<Block>,
        transaction_tx: mpsc::Sender<SignedTransaction>,
        data_dir: &str,
    ) -> Self {
        fs::create_dir_all(data_dir).expect("Can't create data directory");

        Self {
            keypair: get_keypair(data_dir).expect("Can't get keypair"),
            mempool: HashMap::new(),
            db_blocks: storage::db::blocks(false, data_dir),
            db_blocks_metadata: storage::db::blocks_metadata(false, data_dir),
            db_balances: storage::db::balances(false, data_dir),

            block_tx,
            transaction_tx,
        }
    }

    pub fn process_block(&mut self, block: &Block) -> Result<(), String> {
        self.verify_block(block)?;
        self.process_block_transactions(block)?;

        let prev_block_number = storage::get_latest_block_number(&self.db_blocks_metadata)?;
        storage::add_block(&self.db_blocks, block)?;
        storage::set_latest_block_hash(
            &self.db_blocks_metadata,
            &block.hash,
            prev_block_number + 1,
        )?;

        Ok(())
    }

    pub fn add_tx_to_mempool(&mut self, tx: &SignedTransaction) -> Result<(), String> {
        println!(
            "{}:{} - {}={} {}={} {}={} ",
            "New Transaction:".green(),
            tx.tx_id(),
            "amount".yellow(),
            tx.transaction.amount,
            "from".yellow(),
            tx.transaction.from,
            "to".yellow(),
            tx.transaction.to
        );

        self.verify_reg_tx(tx)?;
        self.mempool.insert(tx.tx_id(), tx.clone());

        Ok(())
    }

    pub fn send_tx(&mut self, to: PublicKey, amount: u32) -> Result<SignedTransaction, String> {
        let tx = tx::create_signed(&self.keypair, to, amount);
        self.add_tx_to_mempool(&tx)?;
        self.transaction_tx.send(tx.clone()).unwrap();

        Ok(tx)
    }

    pub fn receive_tx(&mut self, block: &Block) -> Result<(), String> {
        self.process_block(block)?;
        self.block_tx.send(block.clone()).unwrap();

        Ok(())
    }

    pub fn create_coinbase_tx(&self) -> Result<SignedTransaction, String> {
        let latest_block_number = storage::get_latest_block_number(&self.db_blocks_metadata)?;
        let reward = self.get_block_reward(latest_block_number + 1);

        Ok(create_signed(
            &self.keypair,
            self.keypair.public_key,
            reward,
        ))
    }

    pub fn make_gensis_block(&self) -> Result<ProposedBlock, String> {
        let coinbase_tx = self.create_coinbase_tx()?;

        Ok(ProposedBlock {
            prev_block: GENESIS_PREV_BLOCK_HASH.to_string(),
            transactions: vec![coinbase_tx],
        })
    }

    pub fn verify_block(&self, block: &Block) -> Result<(), String> {
        if !block.hash.starts_with(&"0".repeat(DIFFICULTY)) {
            return Err(
                "Block verificatoin failed: Must contains corrent PoW according to difficulty"
                    .to_string(),
            );
        }

        let block_hash = crypto::sha256(block.serialize());
        if hex::encode(block_hash) != block.hash {
            return Err("Block verificatoin failed: Hash mismatch".to_string());
        }

        let prev_block = self.get_latest_block()?;
        let prev_block_hash =
            prev_block.map_or(GENESIS_PREV_BLOCK_HASH.to_string(), |b| b.hash.clone());

        if block.prev_block != prev_block_hash {
            return Err("Block verificatoin failed: Previous block hash mismatch".to_string());
        }

        let prev_block_number = storage::get_latest_block_number(&self.db_blocks_metadata)?;

        for (i, tx) in block.transactions.iter().enumerate() {
            if i == 0 {
                self.verify_coinbase_tx(tx, prev_block_number + 1)?;
            } else {
                self.verify_reg_tx(tx)?;
            }
        }

        // TODO: verify more

        Ok(())
    }

    pub fn process_block_transactions(&mut self, block: &Block) -> Result<(), String> {
        for (i, tx) in block.transactions.iter().enumerate() {
            // Coinbase (first tx in block) is allowed to create new supply (by not deducting a balance)
            if i > 0 {
                let sender_balance = storage::get_balance(&self.db_balances, tx.transaction.from)?
                    .unwrap_or_default();
                let sender_new_balance = sender_balance - tx.transaction.amount;
                storage::set_balance(&self.db_balances, tx.transaction.from, sender_new_balance)?;
            }

            let receiver_balance =
                storage::get_balance(&self.db_balances, tx.transaction.to)?.unwrap_or_default();
            let receiver_new_balance = receiver_balance + tx.transaction.amount;
            storage::set_balance(&self.db_balances, tx.transaction.to, receiver_new_balance)?;

            // Remove tx from mempool
            self.mempool.remove(&tx.transaction.tx_id);
        }

        Ok(())
    }

    pub fn get_latest_block(&self) -> Result<Option<Block>, String> {
        let block_hash = match storage::get_latest_block_hash(&self.db_blocks_metadata)? {
            Some(block_hash) => block_hash,
            None => return Ok(None),
        };

        storage::get_block(&self.db_blocks, &block_hash)
    }

    pub fn get_proposed_block(&mut self) -> Result<ProposedBlock, String> {
        let prev_block = self.get_latest_block().expect("Must have genesis block");

        let block = prev_block.map(|b| {
            let mut txs = vec![self.create_coinbase_tx()?];
            txs.extend(self.mempool.values().cloned());
            Ok(ProposedBlock {
                prev_block: b.hash.clone(),
                transactions: txs,
            })
        });

        block.unwrap_or(self.make_gensis_block())
    }

    pub fn start(&mut self) -> Result<Option<Block>, String> {
        self.get_latest_block()
    }

    pub fn verify_coinbase_tx(
        &self,
        tx: &SignedTransaction,
        block_number: u32,
    ) -> Result<(), String> {
        self.verify_tx(tx)?;
        if tx.transaction.amount != self.get_block_reward(block_number) {
            return Err("Transaction verification failed: Coinbase Amount mismatch".to_string());
        }

        Ok(())
    }

    pub fn verify_reg_tx(&self, tx: &SignedTransaction) -> Result<(), String> {
        self.verify_tx(tx)?;
        let from_balance =
            storage::get_balance(&self.db_balances, tx.transaction.from)?.unwrap_or_default();
        if from_balance < tx.transaction.amount {
            return Err("Transaction verification failed: Insufficient balance".to_string());
        }

        Ok(())
    }

    pub fn verify_tx(&self, tx: &SignedTransaction) -> Result<(), String> {
        if !tx.is_sig_valid() {
            return Err("Transaction verification failed: Invalid signature".to_string());
        }

        Ok(())
    }

    pub fn get_block_reward(&self, block_number: u32) -> u32 {
        let halving = block_number / 1024;
        if halving > 10 {
            return 0;
        }

        512 >> halving
    }
}

pub fn get_keypair(data_dir: &str) -> Result<KeyPair, Box<dyn std::error::Error>> {
    let wallet_path = format!("{}/wallet", data_dir);
    if Path::new(&wallet_path).exists() {
        let key = fs::read_to_string(wallet_path)?;

        return Ok(KeyPair::from(key)?);
    }

    let keypair = KeyPair::new();
    fs::write(
        &wallet_path,
        keypair.private_key.display_secret().to_string(),
    )?;

    Ok(keypair)
}
