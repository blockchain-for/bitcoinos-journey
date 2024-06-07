use std::io::{self, Cursor, ErrorKind, Read};
use std::ops::Add;

#[derive(Debug, Clone, Copy)]
pub struct StandardScripts;

impl StandardScripts {
    pub fn parse(bytes: &mut Cursor<&[u8]>) -> io::Result<String> {
        // Get the first opcode
        let mut opcode_buf = [0u8; 1];
        bytes.read_exact(&mut opcode_buf)?;

        let first_opcode = Opcode::from_byte(opcode_buf[0]);

        match first_opcode {
            // If `OP_PUSHBYTES_65` then parse as P2PK
            Opcode::PushBytes(65) => Self::parse_p2pk(bytes),
            // If `OP_DUP` then parse as P2PKH
            // If `OP_HASH160` then parse as P2SH,
            Opcode::OP_HASH160 => Self::parse_p2sh(bytes),
            // If `OP_RETURN` then parse as Data(OP_RETURN)
            Opcode::OP_DUP => Self::parse_p2pkh(bytes),
            Opcode::OP_RETURN => Self::parse_data(bytes),
            // If `OP_0` as first OP_CODE and second OP_CODE
            // if is OP_PUSHBYTES_20 then parse as P2WPKH
            // else if is OP_PUSHBYTES_32 then parse as P2WSH
            // else return an error
            Opcode::OP_0 => {
                bytes.read_exact(&mut opcode_buf)?;

                let second_opcode = Opcode::from_byte(opcode_buf[0]);

                if second_opcode.eq(&Opcode::PushBytes(20)) {
                    Self::parse_p2wpkh(bytes)
                } else if second_opcode.eq(&Opcode::PushBytes(32)) {
                    Self::parse_p2wsh(bytes)
                } else {
                    return to_io_error(
                        "Invalid Script. Expected OP_PUSHBYTES_20 or OP_PUSHBYTES_32 after OP_0",
                    );
                }
            }
            _ => {
                // If `OP_1` as first OP_CODE and `OP_PUSHBYTES_32` is second OP_CODE then parse as P2TR
                // Else try parsing as P2MS
                bytes.read_exact(&mut opcode_buf)?;
                let second_opcode = Opcode::from_byte(opcode_buf[0]);

                if first_opcode.eq(&Opcode::OP_1) && second_opcode.eq(&Opcode::PushBytes(32)) {
                    Self::parse_p2tr(bytes)
                } else {
                    // Reset current position of cursor to the beginning
                    bytes.set_position(bytes.position() - 2);
                    Self::parse_p2ms(bytes)
                }
            }
        }
    }

    /// Parse as P2PK
    /// A p2pk scriptPubKey looks like:
    /// ASM: OP_PUSHBYTES_65 <pubkey hex> OP_CHECKSIG
    /// e.g. OP_PUSHBYTES_65 0411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3 OP_CHECKSIG
    /// Hex: 410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac
    pub fn parse_p2pk(bytes: &mut Cursor<&[u8]>) -> io::Result<String> {
        // Cursor is already at second byte to parse 65 bytes from data
        // that position to get the uncompressed Public Key
        let mut public_key_bytes = [0u8; 65];
        bytes.read_exact(&mut public_key_bytes)?;

        // Next to parse OP_CHECKSIG
        let mut op_checksig_byte = [0u8; 1];
        bytes.read_exact(&mut op_checksig_byte)?;

        let op_checksig = Opcode::from_byte(op_checksig_byte[0]);

        if op_checksig.ne(&Opcode::OP_CHECKSIG) {
            return to_io_error("Invalid Data. Expected OP_CHECKSIG as last byte of script.");
        }

        // Lastly build the p2pk script
        let mut script_builder = ScriptBuilder::new();
        script_builder
            .push_opcode(Opcode::PushBytes(65))?
            .push_bytes(&public_key_bytes)?
            .push_opcode(Opcode::OP_CHECKSIG)?;

        Ok(script_builder.build())
    }

    /// Parse as P2PKH
    /// A p2pkh scriptPubKey looks like:
    /// ASM: OP_DUP OP_HASH160 OP_PUSHBYTES_20 <pubkey hash hex> OP_EQUALVERIFY OP_CHECKSIG
    /// e.g. OP_DUP OP_HASH160 OP_PUSHBYTES_20 55ae51684c43435da751ac8d2173b2652eb64105 OP_EQUALVERIFY OP_CHECKSIG
    /// Hex: 76a91455ae51684c43435da751ac8d2173b2652eb6410588ac
    pub fn parse_p2pkh(bytes: &mut Cursor<&[u8]>) -> io::Result<String> {
        let mut opcode_buf = [0u8; 1];

        bytes.read_exact(&mut opcode_buf)?;

        // Parse second opcode as OP_HASH160
        let should_be_op_hash160 = Opcode::from_byte(opcode_buf[0]);

        if should_be_op_hash160.ne(&Opcode::OP_HASH160) {
            return to_io_error(
                "Invalid data. Expected OP_HASH160 as second opcode after OP_DUP of the script",
            );
        }

        bytes.read_exact(&mut opcode_buf)?;

        // Parse third opcode as OP_PUSHBYTES_20
        let should_be_op_pushbytes_20 = Opcode::from_byte(opcode_buf[0]);

        if should_be_op_pushbytes_20.ne(&Opcode::PushBytes(20)) {
            return to_io_error(
                "Invalid data. Expected OP_PUSHBYTES_20 as third opcode after OP_HASH160 of the script",
            );
        }

        // Get the 20 bytes of the hash160
        let mut hash160_bytes = [0u8; 20];
        bytes.read_exact(&mut hash160_bytes)?;

        // Parse the next byte as OP_EQUALVERIFY for fourth opcode
        bytes.read_exact(&mut opcode_buf)?;

        let should_be_opequalverify = Opcode::from_byte(opcode_buf[0]);
        if should_be_opequalverify.ne(&Opcode::OP_EQUALVERIFY) {
            return to_io_error(
                "Invalid data, expected OP_EQUALVERIFY as fourth opcode after OP_PUSHBYTES_20 of the script",
            );
        }

        // Parse the next byte as OP_CHECKSIG for fifth opcode
        bytes.read_exact(&mut opcode_buf)?;
        let should_be_opchecksing = Opcode::from_byte(opcode_buf[0]);
        if should_be_opchecksing.ne(&Opcode::OP_CHECKSIG) {
            return to_io_error(
                "Invalid Data. Expected OP_CHECKSIG after reading OP_EQUALVERIFY byte in the script.",
            );
        }

        let mut script_builder = ScriptBuilder::new();
        script_builder
            .push_opcode(Opcode::OP_DUP)?
            .push_opcode(Opcode::OP_HASH160)?
            .push_opcode(Opcode::PushBytes(20))?
            .push_bytes(&hash160_bytes)?
            .push_opcode(Opcode::OP_EQUALVERIFY)?
            .push_opcode(Opcode::OP_CHECKSIG)?;

        Ok(script_builder.build())
    }

    /// Parse as P2SH
    /// A p2sh scriptPubKey looks like:
    /// ASM: OP_HASH160 OP_PUSHBYTES_20 <pubkey hash hex> OP_EQUAL
    /// e.g. OP_HASH160 OP_PUSHBYTES_20 748284390f9e263a4b766a75d0633c50426eb875 OP_EQUAL
    /// Hex: a914748284390f9e263a4b766a75d0633c50426eb87587 in transaction:
    /// 450c309b70fb3f71b63b10ce60af17499bd21b1db39aa47b19bf22166ee67144 (Output 1)
    pub fn parse_p2sh(bytes: &mut Cursor<&[u8]>) -> io::Result<String> {
        let mut opcode_buf = [0u8; 1];
        bytes.read_exact(&mut opcode_buf)?;

        let second_opcode = Opcode::from_byte(opcode_buf[0]);

        // Second opcode should be OP_PUSHBYTES_20
        if second_opcode.ne(&Opcode::PushBytes(20)) {
            return to_io_error(
                "Invalid data. Expected OP_PUSHBYTES_20 as second opcode after OP_HASH160 of the script",
            );
        }

        // Read the 20 bytes of the hash160
        // use read_bytes
        let bytes_20_buf = second_opcode.read_bytes(bytes)?;
        // let mut bytes_20_buf = [0u8; 20];
        // bytes.read_exact(&mut bytes_20_buf)?;

        bytes.read_exact(&mut opcode_buf)?;
        let should_be_opequal = Opcode::from_byte(opcode_buf[0]);
        if should_be_opequal.ne(&Opcode::OP_EQUAL) {
            return to_io_error("Invalid data. Expected OP_EQUAL as the last opcode in the script");
        }

        let mut script_builder = ScriptBuilder::new();
        script_builder
            .push_opcode(Opcode::OP_HASH160)?
            .push_opcode(Opcode::PushBytes(20))?
            .push_bytes(&bytes_20_buf)?
            .push_opcode(Opcode::OP_EQUAL)?;

        Ok(script_builder.build())
    }

    /// Parse as parse_data
    /// An only data output does not have any scriptPubKey, it can't spend and store a little data, it likes:
    /// ASM: OP_RETURN PUSHBYTES_* <data hash hex>
    /// e.g. OP_RETURN PUSHBYTES_11 68656c6c6f20776f726c64
    /// Hex: 6a0b68656c6c6f20776f726c64 in  transaction:
    /// 6dfb16dd580698242bcfd8e433d557ed8c642272a368894de27292a8844a4e75 (Output 2)
    pub fn parse_data(bytes: &mut Cursor<&[u8]>) -> io::Result<String> {
        let mut opcode_buf = [0u8; 1];
        bytes.read_exact(&mut opcode_buf)?;

        let second_opcode = Opcode::from_byte(opcode_buf[0]);

        // Read the number of bytes specified by the opcode
        let data_bytes = second_opcode.read_bytes(bytes)?;

        let mut script_builder = ScriptBuilder::new();
        script_builder
            .push_opcode(Opcode::OP_RETURN)?
            .push_opcode(second_opcode)?
            .push_bytes(&data_bytes)?;

        Ok(script_builder.build())
    }

    /// Parse as P2WPKH
    /// A P2WPKH scriptPubKey looks like:
    /// ASM: OP_0 OP_PUSHBYTES_20 <pubkey hash hex>
    /// e.g. OP_0 OP_PUSHBYTES_20 853ec3166860371ee67b7754ff85e13d7a0d6698
    /// Hex: 0014853ec3166860371ee67b7754ff85e13d7a0d6698
    pub fn parse_p2wpkh(bytes: &mut Cursor<&[u8]>) -> io::Result<String> {
        // Read the next 20 bytes (pubkey hash)
        let mut pubkey_hash_bytes = [0u8; 20];
        bytes.read_exact(&mut pubkey_hash_bytes)?;

        let mut script_builder = ScriptBuilder::new();
        script_builder
            .push_opcode(Opcode::OP_0)?
            .push_opcode(Opcode::PushBytes(20))?
            .push_bytes(&pubkey_hash_bytes)?;

        Ok(script_builder.build())
    }

    /// Parse as P2WSH
    pub fn parse_p2wsh(bytes: &mut Cursor<&[u8]>) -> io::Result<String> {
        todo!()
    }

    /// Parse as P2TR
    pub fn parse_p2tr(bytes: &mut Cursor<&[u8]>) -> io::Result<String> {
        todo!()
    }

    /// Parse as P2MS
    pub fn parse_p2ms(bytes: &mut Cursor<&[u8]>) -> io::Result<String> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
pub enum Opcode {
    OP_0, // 0|0x00
    /// Handels all OP_PUSHBYTES_*
    PushBytes(u8), // 1-75|0x01-0x4b
    OP_1, // 81|0x51
    /// Handles OP_2 TO OP_16
    Num(u8), // 82-96|0x52-0x60

    OP_RETURN,        // 106|0x6a
    OP_DUP,           // 118|0x76
    OP_EQUAL,         // 135|0x87
    OP_EQUALVERIFY,   // 136|0x88
    OP_HASH160,       // 169|0xa9
    OP_CHECKSIG,      // 172|0xac
    OP_CHECKMULTISIG, // 174|0xae

    /// Useful in error handling for unsupported opcodes
    UnsupportedOpcode,
}

impl Opcode {
    /// Parse an opcode from a hex decoded byte
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0 => Self::OP_0,
            // All OP_PUSHBYTES_*
            1..=75 => Self::PushBytes(byte),
            81 => Self::OP_1,
            // All OP_2 - OP_16
            82..=96 => {
                let to_num = match byte {
                    82 => 2u8,
                    83 => 3,
                    84 => 4,
                    85 => 5,
                    86 => 6,
                    87 => 7,
                    88 => 8,
                    89 => 9,
                    90 => 10,
                    91 => 11,
                    92 => 12,
                    93 => 13,
                    94 => 14,
                    95 => 15,
                    96 => 16,
                    _ => return Self::UnsupportedOpcode,
                };
                Self::Num(to_num)
            }
            106 => Self::OP_RETURN,
            118 => Self::OP_DUP,
            135 => Self::OP_EQUAL,
            136 => Self::OP_EQUALVERIFY,

            169 => Self::OP_HASH160,
            172 => Self::OP_CHECKSIG,
            174 => Self::OP_CHECKMULTISIG,
            _ => Self::UnsupportedOpcode,
        }
    }

    /// Handles reading OP_PUSHBYTES_*
    pub fn read_bytes(&self, bytes: &mut Cursor<&[u8]>) -> io::Result<Vec<u8>> {
        // Store all parsed bytes for `OP_PUSHBYES_*`
        let mut buffer = Vec::new();

        match self {
            Self::PushBytes(byte_len) => {
                // Gets the current position and adds the length of the opcode
                let new_position = (bytes.position() as usize).add(*byte_len as usize);
                // Read the byte  slice from the current cursor position the byte length
                buffer.extend_from_slice(&bytes.get_ref()[bytes.position() as usize..new_position]);
                // Set the cursor position to the previous cursor position + the byte length
                bytes.set_position(new_position as u64);

                Ok(buffer)
            }
            _ => Err(io::Error::new(
                ErrorKind::Unsupported,
                "This operation is not supported",
            )),
        }
    }
}

impl TryFrom<Opcode> for String {
    type Error = io::Error;

    fn try_from(value: Opcode) -> Result<Self, Self::Error> {
        let opcode = match value {
            Opcode::OP_0 => "OP_0",
            Opcode::PushBytes(v) => {
                return Ok(String::from("OP_PUSHBYTES_").add(v.to_string().as_str()))
            }
            Opcode::OP_1 => "OP_1",
            Opcode::Num(num) => return Ok(String::from("OP_{}").add(num.to_string().as_str())),
            Opcode::OP_RETURN => "OP_RETURN",
            Opcode::OP_DUP => "OP_DUP",
            Opcode::OP_EQUAL => "OP_EQUAL",
            Opcode::OP_EQUALVERIFY => "OP_EQUALVERIFY",
            Opcode::OP_HASH160 => "OP_HASH160",
            Opcode::OP_CHECKSIG => "OP_CHECKSIG",
            Opcode::OP_CHECKMULTISIG => "OP_CHECKMULTISIG",
            Opcode::UnsupportedOpcode => {
                return Err(io::Error::new(
                    ErrorKind::Unsupported,
                    "This operation is not supported",
                ))
            }
        };

        Ok(opcode.into())
    }
}

#[derive(Debug, Default)]
pub struct ScriptBuilder(Vec<String>);

impl ScriptBuilder {
    /// Initialize `Self` with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Receive an `Opcode` and convert it into a String
    pub fn push_opcode(&mut self, opcode: Opcode) -> io::Result<&mut Self> {
        let opcode_str: String = opcode.try_into()?;
        self.0.push(opcode_str);

        Ok(self)
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) -> io::Result<&mut Self> {
        self.0.push(hex::encode(bytes));

        Ok(self)
    }

    pub fn build(self) -> String {
        self.0
            .into_iter()
            .map(|mut part| {
                part.push(' ');
                part
            })
            .collect::<String>()
            .trim()
            .into()
    }
}

/// Error handling returning an `io::Result<String>` to avoid having to add `Err()`
/// whenever calling this method.
pub fn to_io_error(message: &str) -> io::Result<String> {
    Err(io::Error::new(ErrorKind::InvalidData, message))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::*;

    #[test]
    fn parse_p2pk_should_work() {
        let p2pk_bytes = hex!("410000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000ac");
        let mut p2pk_cursor = Cursor::new(p2pk_bytes.as_ref());
        let outcome = StandardScripts::parse(&mut p2pk_cursor);

        assert!(outcome.is_ok());

        dbg!(&outcome.unwrap());
    }

    #[test]
    fn parse_p2pkh_should_work() {
        let p2pkh_byts = hex!("76a914000000000000000000000000000000000000000088ac");
        let mut p2pkh_cursor = Cursor::new(p2pkh_byts.as_ref());
        let outcome = StandardScripts::parse(&mut p2pkh_cursor);

        assert!(outcome.is_ok());

        dbg!(&outcome.unwrap());
    }

    #[test]
    fn parse_p2sh_should_work() {
        let p2sh_bytes = hex!("a914f54a6690e0fb67c222aafde6482a66eeb74ebf5c87");
        let mut p2sh = Cursor::new(p2sh_bytes.as_ref());
        let outcome = StandardScripts::parse(&mut p2sh);

        assert!(outcome.is_ok());

        dbg!(&outcome.unwrap());
    }

    #[test]
    fn parse_p2sh_should_work_2() {
        let p2sh_bytes = hex!("a914748284390f9e263a4b766a75d0633c50426eb87587");
        let mut p2sh = Cursor::new(p2sh_bytes.as_ref());
        let outcome = StandardScripts::parse(&mut p2sh);

        assert!(outcome.is_ok());

        dbg!(&outcome.unwrap());
    }

    #[test]
    fn parse_data_should_work() {
        let on_return_bytes = hex!("6a0b68656c6c6f20776f726c64");
        let mut on_return_cursor = Cursor::new(on_return_bytes.as_ref());
        let outcome = StandardScripts::parse(&mut on_return_cursor);

        assert!(outcome.is_ok());

        dbg!(&outcome.unwrap());
    }

    #[test]
    fn parse_p2wpkh_should_work() {
        let p2wpkh_bytes = hex!("0014751e76e8199196d454941c45d1b3a323f1433bd6");
        let mut p2wpkh = Cursor::new(p2wpkh_bytes.as_ref());
        let outcome = StandardScripts::parse(&mut p2wpkh);

        assert!(outcome.is_ok());

        dbg!(&outcome.unwrap());
    }
}
