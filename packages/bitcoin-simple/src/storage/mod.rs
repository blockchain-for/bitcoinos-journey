use crate::block::Block;

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
