use crate::varint::VarInt;
use crate::version::TxVersion;

use std::io::{self, Cursor, Read};

/// Transaction Input
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct TxInput {
    // The SHA256 bytes of the previous transaction ID of the unspent output
    previous_tx_id: [u8; 32],
    // Previous output index
    previous_output_index: u32,
    // The scriptSig of the input
    signature_script: Vec<u8>,
    // The sequence number of the input
    sequence_number: u32,
}

/// Transaction Output
#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct TxOutput {
    // Amount in Satoshis
    amount_in_satoshi: u64,
    // The locking script which gives conditions to spend this output
    locking_script: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct BtcTx {
    // The version of the Bitcoin transaction
    version: TxVersion,
    // A transaction can have multiple inputs
    inputs: Vec<TxInput>,
    // A transaction can have multiple outputs
    outputs: Vec<TxOutput>,
    // The locktime of the transaction, from 4 bytes into u32
    locktime: u32,
}

impl BtcTx {
    // Convert hex bytes into a Transaction struct.
    // This calls all other methods to parse the version, inputs, outputs and locktime
    pub fn from_hex_bytes(bytes: impl AsRef<[u8]>) -> io::Result<Self> {
        // Instantiate a new cursor to hold the bytes
        // The cursor's position advances where we read bytes allowing us to simplify the logic
        // instead of using a counter to keep track of bytes read
        let mut bytes = Cursor::new(bytes.as_ref());

        // the version is always the first 4 byte array
        let mut version_bytes = [0u8; 4];
        // Read exactly 4 bytes and advance the cursor to the 4th byte
        bytes.read_exact(&mut version_bytes)?;
        // Get the transaction version from the bytes
        let version = TxVersion::from_bytes(version_bytes);

        // Get a vector of inputs by calling the `Self::get_inputs()` method
        let inputs = BtcTx::get_inputs(&mut bytes)?;

        let outputs = BtcTx::get_outputs(&mut bytes)?;

        let locktime = BtcTx::locktime(&mut bytes)?;

        Ok(BtcTx {
            version,
            inputs,
            outputs,
            locktime,
        })
    }

    /// Get all inputs from the current position of the `Cursor`.
    /// This method decodes the number of inputs by first decoding the `VarInt` and then
    /// looping number of inputs calling  `Self::input_decoder()` on each iteration.
    fn get_inputs(bytes: &mut Cursor<&[u8]>) -> io::Result<Vec<TxInput>> {
        let mut varint_len = [0u8];
        bytes.read_exact(&mut varint_len)?;

        let varint_byte_len = VarInt::parse(varint_len[0]);
        let num_of_inputs = VarInt::integer(varint_byte_len, bytes)?;

        let mut inputs = Vec::<TxInput>::new();

        (0..num_of_inputs).for_each(|_| {
            inputs.push(BtcTx::input_decoder(bytes).unwrap());
        });

        Ok(inputs)
    }

    /// Get the outputs after all inputs have been parsed
    fn get_outputs(bytes: &mut Cursor<&[u8]>) -> io::Result<Vec<TxOutput>> {
        // Get the numberof outputs by reading our varint
        let mut number_of_output_bytes = [0u8];
        bytes.read_exact(&mut number_of_output_bytes)?;

        let var_int_byte_len = VarInt::parse(number_of_output_bytes[0]);
        // Convert varint to an integer
        let num_of_outputs = VarInt::integer(var_int_byte_len, bytes)?;

        let mut outputs: Vec<TxOutput> = Vec::new();

        (0..num_of_outputs).for_each(|_| {
            outputs.push(BtcTx::output_decoder(bytes).unwrap());
        });

        Ok(outputs)
    }

    // Lastly, after parsing our version, inputs, outputs, we parse the locktime
    fn locktime(bytes: &mut Cursor<&[u8]>) -> io::Result<u32> {
        // Locktime is 4 bytes long
        let mut locktime_bytes = [0u8; 4];
        bytes.read_exact(&mut locktime_bytes)?;

        // Convert the locktime to a u32
        Ok(u32::from_le_bytes(locktime_bytes))
    }

    fn input_decoder(bytes: &mut Cursor<&[u8]>) -> io::Result<TxInput> {
        // The previous transaction ID is always a SHA256 hash converted to a 32 byte array
        let mut previous_tx_id = [0u8; 32];

        // Read exactly 32 bytes and advance the cursor to the end of the 32 byte array
        bytes.read_exact(&mut previous_tx_id)?;

        // The transaction ID in hex format is in network byte order
        // So we reverse it to little endian
        previous_tx_id.reverse();

        // Previous transaction index (vout) is 4 bytes long which is a Rust u32
        let mut previous_tx_index_bytes = [0u8; 4];
        bytes.read_exact(&mut previous_tx_index_bytes)?;

        // Convert the read 4 bytes to a u32
        let previous_output_index = u32::from_le_bytes(previous_tx_index_bytes);

        // Get the length of the scriptSig
        let mut script_sig_size = [0u8];

        bytes.read_exact(&mut script_sig_size)?;

        // Parse the length VarInt
        let varint_byte_len = VarInt::parse(script_sig_size[0]);

        // Get the length by converting VarInt into a integer by call `integer`
        let integer_from_varint = VarInt::integer(varint_byte_len, bytes)?;

        // Buffer to hold the signature script
        let mut signature_script: Vec<u8> = vec![];

        let mut sig_buf = [0u8; 1];

        // Since we are using a cursor, we iterate in order to advance the cursor in each iteration
        (0..integer_from_varint).for_each(|idx| {
            println!("Signature buffer: {sig_buf:?} before {idx} iteration");
            bytes.read_exact(&mut sig_buf).unwrap();
            println!("Signature buffer: {sig_buf:?} in {idx} iteration");
            signature_script.extend_from_slice(&sig_buf);
            println!("Signature buffer: {sig_buf:?} after {idx} iteration");
        });

        // The sequence number is a u32 (4 bytes long)
        let mut sequence_num_bytes = [0u8; 4];
        bytes.read_exact(&mut sequence_num_bytes)?;

        // Convert the sequence number to a integer
        let sequence_number = u32::from_le_bytes(sequence_num_bytes);

        Ok(TxInput {
            previous_tx_id,
            previous_output_index,
            signature_script,
            sequence_number,
        })
    }

    fn output_decoder(bytes: &mut Cursor<&[u8]>) -> io::Result<TxOutput> {
        // The first value of the output is the amount in satoshis
        // which is 8 bytes long (Rust u64)
        let mut amount_in_satoshi_bytes = [0u8; 8];
        bytes.read_exact(&mut amount_in_satoshi_bytes)?;

        // Get the number of satoshis in decimal
        let amount_in_satoshi = u64::from_le_bytes(amount_in_satoshi_bytes);

        // Get the exact size of the locking script
        let mut locking_script_len = [0u8; 1];
        bytes.read_exact(&mut locking_script_len)?;

        // Parse the length into a varint
        let script_byte_len = VarInt::parse(locking_script_len[0]);

        // Convert our VarInt to an integer
        let script_len = VarInt::integer(script_byte_len, bytes)?;

        let mut locking_script: Vec<u8> = vec![];

        (0..script_len).for_each(|idx| {
            let mut current_byte = [0u8];
            println!("locking script  buffer: {current_byte:?} before {idx} iteration");

            bytes.read_exact(&mut current_byte).unwrap();
            println!("locking script buffer: {current_byte:?} in {idx} iteration");

            locking_script.extend_from_slice(&current_byte);

            println!("locking script buffer: {current_byte:?} after {idx} iteration");
        });

        Ok(TxOutput {
            amount_in_satoshi,
            locking_script,
        })
    }
}
