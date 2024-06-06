#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TxVersion {
    #[default]
    One,
    Two,
    Custome(u32),
}

impl TxVersion {
    pub fn to_bytes(&self) -> [u8; 4] {
        match self {
            Self::One => 1u32.to_le_bytes(),
            Self::Two => 2u32.to_le_bytes(),
            Self::Custome(v) => v.to_le_bytes(),
        }
    }

    pub fn from_bytes(bytes: [u8; 4]) -> Self {
        let parsed = u32::from_le_bytes(bytes);

        match parsed {
            1u32 => Self::One,
            2u32 => Self::Two,
            _ => Self::Custome(parsed),
        }
    }
}

#[cfg(test)]
mod tx_sanity_checks {
    use super::*;

    #[test]
    fn tx_version_should_works() {
        assert_eq!([1u8, 0, 0, 0], TxVersion::One.to_bytes());
        assert_eq!([2u8, 0, 0, 0], TxVersion::Two.to_bytes());
        assert_eq!([33u8, 0, 0, 0], TxVersion::Custome(33).to_bytes());

        assert_eq!(TxVersion::One, TxVersion::from_bytes([1u8, 0, 0, 0]));
        assert_eq!(TxVersion::Two, TxVersion::from_bytes([2u8, 0, 0, 0]));
        assert_eq!(
            TxVersion::Custome(36),
            TxVersion::from_bytes([36u8, 0, 0, 0])
        );
    }
}
