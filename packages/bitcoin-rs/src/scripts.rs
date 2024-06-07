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
    /// A P2WSH scriptPubKey looks like:
    /// ASM: OP_0 OP_PUSHBYTES_32 <pubkey hash hex>
    /// e.g. : OP_0 OP_PUSHBYTES_32 65f91a53cb7120057db3d378bd0f7d944167d43a7dcbff15d6afc4823f1d3ed3
    /// Hex: 002065f91a53cb7120057db3d378bd0f7d944167d43a7dcbff15d6afc4823f1d3ed3
    /// in Transaction: 46ebe264b0115a439732554b2b390b11b332b5b5692958b1754aa0ee57b64265 (Output 1)
    pub fn parse_p2wsh(bytes: &mut Cursor<&[u8]>) -> io::Result<String> {
        // Already parse the first opcode as OP_0 and the second opcode as OP_PUSHBYTES_32
        // Parse next 32 bytes
        let mut hash_bytes = [0u8; 32];
        bytes.read_exact(&mut hash_bytes)?;

        let mut script_builder = ScriptBuilder::new();
        script_builder
            .push_opcode(Opcode::OP_0)?
            .push_opcode(Opcode::PushBytes(32))?
            .push_bytes(&hash_bytes)?;

        Ok(script_builder.build())
    }

    /// Parse as P2TR
    pub fn parse_p2tr(bytes: &mut Cursor<&[u8]>) -> io::Result<String> {
        // Already parse the first opcode as OP_1 and the second opcode as OP_PUSHBYTES_32
        // Parse next 32 bytes
        let mut hash_bytes = [0u8; 32];
        bytes.read_exact(&mut hash_bytes)?;

        let mut script_builder = ScriptBuilder::new();
        script_builder
            .push_opcode(Opcode::Num(1))?
            .push_opcode(Opcode::PushBytes(32))?
            .push_bytes(&hash_bytes)?;

        Ok(script_builder.build())
    }

    /// Parse as P2MS
    /// Also checks to see if the number of public keys parsed is equal to number of public keys requires
    /// or if the parsed public keys  are less than the threshold
    /// A P2MS scriptPubKey looks like:
    /// ASM: OP_2 OP_PUSHBYTES_65 <pubkey1 hash hex> OP_PUSHBYTES_65 <pubkey2 hash hex> OP_PUSHBYTES_65 <pubkey3 hash hex> OP_3 OP_CHECKMULTISIG
    /// e.g. : OP_2 OP_PUSHBYTES_65 04d81fd577272bbe73308c93009eec5dc9fc319fc1ee2e7066e17220a5d47a18314578be2faea34b9f1f8ca078f8621acd4bc22897b03daa422b9bf56646b342a2 \
    /// OP_PUSHBYTES_65 04ec3afff0b2b66e8152e9018fe3be3fc92b30bf886b3487a525997d00fd9da2d012dce5d5275854adc3106572a5d1e12d4211b228429f5a7b2f7ba92eb0475bb1 \
    /// OP_PUSHBYTES_65 04b49b496684b02855bc32f5daefa2e2e406db4418f3b86bca5195600951c7d918cdbe5e6d3736ec2abf2dd7610995c3086976b2c0c7b4e459d10b34a316d5a5e7 \
    /// OP_3 OP_CHECKMULTISIG
    /// Hex: 524104d81fd577272bbe73308c93009eec5dc9fc319fc1ee2e7066e17220a5d47a18314578be2faea34b9f1f8ca078f8621acd4bc22897b03daa422b9bf56646b342a24104ec3afff0b2b66e8152e9018fe3be3fc92b30bf886b3487a525997d00fd9da2d012dce5d5275854adc3106572a5d1e12d4211b228429f5a7b2f7ba92eb0475bb14104b49b496684b02855bc32f5daefa2e2e406db4418f3b86bca5195600951c7d918cdbe5e6d3736ec2abf2dd7610995c3086976b2c0c7b4e459d10b34a316d5a5e753ae
    pub fn parse_p2ms(bytes: &mut Cursor<&[u8]>) -> io::Result<String> {
        let mut opcode_buf = [0u8; 1];
        bytes.read_exact(&mut opcode_buf)?;

        let threshold_opcode = Opcode::from_byte(opcode_buf[0]);

        match threshold_opcode {
            Opcode::Num(_) | Opcode::OP_1 => {
                let mut script_builder = ScriptBuilder::new();
                script_builder.push_opcode(threshold_opcode)?;

                // The number of public keys parsed
                let mut pubkey_count = 0u8;
                // the number of public keys specified in the scriptSig
                let parsed_pubkey_count: u8;
                let mut pushbytes_buf: Vec<u8> = Vec::new();

                loop {
                    bytes.read_exact(&mut opcode_buf)?;
                    let current_opcode = Opcode::from_byte(opcode_buf[0]);

                    match current_opcode {
                        Opcode::Num(v) => {
                            parsed_pubkey_count = v;
                            script_builder.push_opcode(current_opcode)?;

                            // Break the loop if a `OP_1 to OP_16` is encountered
                            break;
                        }
                        Opcode::PushBytes(v) => {
                            let new_position = bytes.position() as usize + v as usize;
                            let read_bytes =
                                &bytes.get_ref()[bytes.position() as usize..new_position];

                            pushbytes_buf.extend_from_slice(read_bytes);

                            script_builder
                                .push_opcode(current_opcode)?
                                .push_bytes(&pushbytes_buf)?;

                            pushbytes_buf.clear();
                            bytes.set_position(new_position as u64);
                            pubkey_count += 1;
                        }
                        _ => {
                            return to_io_error(
                                "invalid Script. Expected a PUSHBYTES_* or OP_1 to OP_16",
                            );
                        }
                    }
                }

                if pubkey_count.ne(&parsed_pubkey_count) {
                    return to_io_error("Invalid Script. The number of public keys for multisignagure is less than or greater than the script requirements.");
                }

                if let Opcode::Num(threshold_inner) = threshold_opcode {
                    if parsed_pubkey_count.lt(&threshold_inner) {
                        return to_io_error("Invalid script, the number of public keys for multisignagure is less than the threshold.");
                    }
                }

                // Parse next byte and check if it is OP_CHECKMULTISIG opcode
                bytes.read_exact(&mut opcode_buf)?;

                let opcheck_multisig = Opcode::from_byte(opcode_buf[0]);

                if opcheck_multisig.ne(&Opcode::OP_CHECKMULTISIG) {
                    return to_io_error("Invalid Script. OP_CHECKMULTISIG opcode should be next ");
                }

                script_builder.push_opcode(Opcode::OP_CHECKMULTISIG)?;

                Ok(script_builder.build())
            }
            _ => to_io_error("Invalid script."),
        }
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
            Opcode::Num(num) => return Ok(String::from("OP_").add(num.to_string().as_str())),
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

    #[test]
    fn parse_p2wsh_should_works() {
        let p2wsh_bytes =
            hex!("002065f91a53cb7120057db3d378bd0f7d944167d43a7dcbff15d6afc4823f1d3ed3");
        let mut p2wsh = Cursor::new(p2wsh_bytes.as_ref());
        let outcome = StandardScripts::parse(&mut p2wsh);

        assert!(outcome.is_ok());

        dbg!(&outcome.unwrap());
    }

    #[test]
    fn parse_p2tr_should_work() {
        let p2tr_bytes =
            hex!("51200000000000000000000000000000000000000000000000000000000000000000");
        let mut p2tr = Cursor::new(p2tr_bytes.as_ref());
        let outcome = StandardScripts::parse(&mut p2tr);

        assert!(outcome.is_ok());

        dbg!(&outcome.unwrap());
    }

    #[test]
    fn parse_p2ms_2x3_should_work() {
        let p2ms_bytes = hex!("524104d81fd577272bbe73308c93009eec5dc9fc319fc1ee2e7066e17220a5d47a18314578be2faea34b9f1f8ca078f8621acd4bc22897b03daa422b9bf56646b342a24104ec3afff0b2b66e8152e9018fe3be3fc92b30bf886b3487a525997d00fd9da2d012dce5d5275854adc3106572a5d1e12d4211b228429f5a7b2f7ba92eb0475bb14104b49b496684b02855bc32f5daefa2e2e406db4418f3b86bca5195600951c7d918cdbe5e6d3736ec2abf2dd7610995c3086976b2c0c7b4e459d10b34a316d5a5e753ae");
        let mut p2ms_cursor = Cursor::new(p2ms_bytes.as_ref());
        let outcome = StandardScripts::parse(&mut p2ms_cursor);

        assert!(outcome.is_ok());

        dbg!(&outcome.unwrap());
    }

    #[test]
    fn parse_p2ms_1x2_should_work() {
        let p2ms_2_bytes = hex!("51210000000000000000000000000000000000000000000000000000000000000000002100000000000000000000000000000000000000000000000000000000000000000052ae");
        let mut p2ms_2 = Cursor::new(p2ms_2_bytes.as_ref());
        let outcome = StandardScripts::parse(&mut p2ms_2);

        assert!(outcome.is_ok());

        dbg!(&outcome.unwrap());
    }
}
