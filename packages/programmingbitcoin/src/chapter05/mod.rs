use std::fmt::Display;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tx {
    version: u64,
    tx_ins: Vec<String>,
    tx_outs: Vec<String>,
    locktime: u64,  
    testnet: bool,
}

impl Tx {
    pub fn id(&self) -> String {
        // hash and hex
        self.hash()
    }

    pub fn hash(&self) -> String {
        // hash256
        todo!()
    }
}

impl Display for Tx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, 
            "tx: {}\nversion: {}\n tx_ins: {:?}\n tx_outs: {:?}\n locktime: {}\n",
            self.id(), self.version, self.tx_ins, self.tx_outs, self.locktime
        )
    }
}