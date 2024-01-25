pub mod db;

pub type Store = rocksdb::DB;

pub fn get_block_hashes(db: &Store) -> Result<Vec<String>, String> {
    todo!()
}
