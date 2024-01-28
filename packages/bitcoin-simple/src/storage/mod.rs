use std::collections::HashMap;

use std::str::FromStr;

use crate::{block::Block, crypto::key::PublicKey};

pub mod db;

pub type Store = rocksdb::DB;

pub fn get_block_hashes(db: &Store) -> Result<Vec<String>, String> {
    let mut blocks = Vec::new();
    let mut iter = db.raw_iterator();
    iter.seek_to_first();

    while iter.valid() {
        let block_hash =
            String::from_utf8(iter.key().unwrap().to_vec()).map_err(|e| e.to_string())?;

        if block_hash != "latest_block_hash" {
            let block_number_s =
                String::from_utf8(iter.value().unwrap().to_vec()).map_err(|e| e.to_string())?;
            if let Ok(block_number) = block_number_s.parse::<u32>() {
                blocks.push((block_number, block_hash))
            }
        }

        iter.next();
    }

    blocks.sort_unstable_by(|a, b| a.0.cmp(&b.0));

    let block_hashes = blocks.iter().map(|x| x.1.clone()).collect();

    Ok(block_hashes)
}

pub fn get_block(db: &Store, block_hash: &str) -> Result<Option<Block>, String> {
    match db.get(block_hash)? {
        Some(block) => {
            let block_str = String::from_utf8(block).map_err(|e| e.to_string())?;
            serde_json::from_str(&block_str).map_err(|e| e.to_string())
        }
        None => Ok(None),
    }
}

pub fn get_latest_block_hash(db: &Store) -> Result<Option<String>, String> {
    db.get(b"latest_block_hash")
        .map(|hash| hash.map(|b| String::from_utf8(b).unwrap()))
        .map_err(|e| e.to_string())
}

pub fn add_block(db: &Store, block: &Block) -> Result<(), String> {
    let block_json = serde_json::to_string(&block).map_err(|e| e.to_string())?;
    db.put(block.hash.clone(), block_json)
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn set_latest_block_hash(db: &Store, block_hash: &str, height: u32) -> Result<(), String> {
    db.put(b"latest_block_hash", block_hash)
        .map_err(|e| e.to_string())?;
    db.put(block_hash, height.to_string())
        .map_err(|e| e.to_string())?;
    db.put(height.to_string(), block_hash)
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn get_block_height(db: &Store, block: &str) -> Result<Option<u32>, String> {
    db.get(block)
        .map(|hash| hash.and_then(|b| String::from_utf8(b).unwrap().parse::<u32>().ok()))
        .map_err(|e| e.to_string())
}

pub fn set_balance(db: &Store, public_key: PublicKey, balance: u32) -> Result<(), String> {
    db.put(public_key.to_string(), balance.to_string())
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn get_balance(db: &Store, public_key: PublicKey) -> Result<Option<u32>, String> {
    db.get(public_key.to_string())
        .map(|hash| hash.and_then(|b| String::from_utf8(b).unwrap().parse::<u32>().ok()))
        .map_err(|e| e.to_string())
}

pub fn get_balances(db: &Store) -> Result<HashMap<PublicKey, u32>, String> {
    let mut balances = HashMap::new();
    let mut iter = db.raw_iterator();
    iter.seek_to_first();

    while iter.valid() {
        let public_key =
            String::from_utf8(iter.key().unwrap().to_vec()).map_err(|e| e.to_string())?;
        let balance =
            String::from_utf8(iter.value().unwrap().to_vec()).map_err(|e| e.to_string())?;
        let balance = balance.parse::<u32>().unwrap();

        balances.insert(PublicKey::from_str(&public_key).unwrap(), balance);
        iter.next();
    }

    Ok(balances)
}

pub fn get_latest_block_number(db: &Store) -> Result<u32, String> {
    let latest_block_hash = match get_latest_block_hash(db)? {
        Some(hash) => hash,
        None => return Ok(0),
    };

    get_block_height(db, &latest_block_hash).map(|h| h.unwrap_or_default())
}
