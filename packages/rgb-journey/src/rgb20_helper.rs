// use std::{convert::Infallible, fs};

// use amplify::hex::FromHex;
// use bp::{Chain, Outpoint, Tx, Txid};
// use rgb_schemata::{nia_rgb20, nia_schema};
// use rgbstd::containers::BindleContent;
// use rgbstd::interface::{FungibleAllocation, Rgb20};
// use rgbstd::{
//     contract::WitnessOrd,
//     interface::{rgb20, ContractBuilder},
//     persistence::{Inventory, Stock},
//     resolvers::ResolveHeight,
//     stl::{Amount, ContractData, DivisibleAssetSpec, Precision, RicardianContract, Timestamp},
//     validation::{ResolveTx, TxResolverError},
// };
// use strict_encoding::StrictDumb;

// struct DumbResolver {}

// impl ResolveTx for DumbResolver {
//     fn resolve_tx(&self, _txid: Txid) -> Result<Tx, TxResolverError> {
//         Ok(Tx::strict_dumb())
//     }
// }

// impl ResolveHeight for DumbResolver {
//     type Error = Infallible;

//     fn resolve_height(&mut self, _txid: Txid) -> Result<rgbstd::contract::WitnessOrd, Self::Error> {
//         Ok(WitnessOrd::OffChain)
//     }
// }

// pub fn create_and_validate() {
//     let spec = DivisibleAssetSpec::new("BOS", "Bitcoin OS", Precision::CentiMicro);
//     let terms = RicardianContract::default();
//     let contract_data = ContractData { terms, media: None };

//     let created_time = Timestamp::now();
//     let beneficiary = Outpoint::new(
//         Txid::from_hex("53a03aac89c27d6161ca1de451d8f260590717a185e2741d15bddef0f6928f96").unwrap(),
//         1,
//     );

//     const ISSUE: u64 = 100_000_000_000;

//     let contract = ContractBuilder::with(rgb20(), nia_schema(), nia_rgb20())
//         .expect("schema fails to implement RGB20 interface")
//         .set_chain(Chain::Testnet3)
//         .add_global_state("spec", spec)
//         .expect("invalid nominal")
//         .add_global_state("created", created_time)
//         .expect("invalid created time")
//         .add_global_state("data", contract_data)
//         .expect("invalid contract data")
//         .add_global_state("issuedSupply", Amount::from(ISSUE))
//         .expect("invalid issued supply")
//         .add_fungible_state("assetOwner", beneficiary, ISSUE)
//         .expect("invalid asset amount")
//         .issue_contract()
//         .expect("contract doestn't fit schema requirements");

//     let contract_id = contract.contract_id();
//     println!("contract id: {contract_id}");
//     debug_assert_eq!(contract_id, contract.contract_id());

//     let bindle = contract.bindle();
//     eprintln!("{bindle}");

//     bindle
//         .save("rgbs/rgb20-bos.contract.rgb")
//         .expect("unable to save contract");
//     fs::write("rgbs/rgb20-bos.contract.rgba", bindle.to_string())
//         .expect("unable to save contract with fs");

//     // Create some stock -- an in-memory stash and inventory
//     let mut stock = Stock::default();

//     stock.import_iface(rgb20()).unwrap();
//     stock.import_schema(nia_schema()).unwrap();
//     stock.import_iface_impl(nia_rgb20()).unwrap();

//     // verify contract consignment and add it to the stock
//     let verified_contract = match bindle.unbindle().validate(&mut DumbResolver {}) {
//         Ok(consignment) => consignment,
//         Err(consignment) => panic!(
//             "Can't produce valid consignment. Report: {}",
//             consignment
//                 .validation_status()
//                 .expect("status always present upon validation")
//         ),
//     };

//     stock
//         .import_contract(verified_contract, &mut DumbResolver {})
//         .unwrap();

//     // Reading contract state throught the interface from the stock
//     let contract = stock
//         .contract_iface(contract_id, rgb20().iface_id())
//         .unwrap();
//     let contract = Rgb20::from(contract);
//     let allocations = contract.fungible("assetOwner", &None).unwrap();
//     eprintln!("{}", serde_json::to_string(&contract.spec()).unwrap());

//     for FungibleAllocation {
//         owner,
//         witness,
//         value,
//     } in allocations
//     {
//         eprintln!("amount: {value}, owner: {owner}, witness: {witness}");
//     }

//     eprintln!("totalSupply: {}", contract.total_supply());
//     eprintln!("created: {:?}", contract.created().to_local().unwrap());
// }
