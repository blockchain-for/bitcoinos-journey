use rocksdb::Options;

use super::Store;

static BLOCKS_DB_PATH: &str = "./blocks";
static BLOCKS_METADATA_DB_PATH: &str = "./blocksmetadata";
static BALANCES_DB_PATH: &str = "./balances";

pub fn blocks(read_only: bool, data_dir: &str) -> Store {
    open(BLOCKS_DB_PATH, read_only, data_dir)
}

pub fn blocks_metadata(read_only: bool, data_dir: &str) -> Store {
    open(BLOCKS_METADATA_DB_PATH, read_only, data_dir)
}

pub fn balances(read_only: bool, data_dir: &str) -> Store {
    open(BALANCES_DB_PATH, read_only, data_dir)
}

fn open(path: &str, read_only: bool, data_dir: &str) -> Store {
    let full_path = format!("{}{}", data_dir, path);

    if read_only {
        Store::open_for_read_only(&Options::default(), full_path, true).unwrap()
    } else {
        Store::open_default(full_path).unwrap()
    }
}
