use std::io::{Error, ErrorKind};

use sha2::{Digest, Sha256};

use transaction::{Amount, Input, Output, Transaction, Txid};

mod transaction;

#[allow(dead_code)]
fn read_version(transaction_hex: &str) -> u32 {
    let bytes = match hex_to_bytes(transaction_hex) {
        Ok(bytes) => bytes,
        Err(_) => return 0,
    };

    if bytes.len() < 4 {
        return 0;
    }

    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, Error> {
    if !hex.len().is_multiple_of(2) {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "transaction hex must have an even number of characters",
        ));
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);

    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| {
            Error::new(
                ErrorKind::InvalidInput,
                "transaction contains invalid hexadecimal",
            )
        })?;

        bytes.push(byte);
    }

    Ok(bytes)
}

fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    if transaction_bytes.len() < 8 {
        return 0;
    }

    let (value, rest) = transaction_bytes.split_at(8);
    *transaction_bytes = rest;

    u64::from_le_bytes([
        value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7],
    ])
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    if transaction_bytes.len() < 8 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "not enough bytes to read amount",
        ));
    }

    Ok(Amount::from_sat(read_u64(transaction_bytes)))
}

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    if bytes_slice.len() < 4 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "not enough bytes to read u32",
        ));
    }

    let (value, rest) = bytes_slice.split_at(4);
    *bytes_slice = rest;

    Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
}

fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    if transaction_bytes.is_empty() {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "not enough bytes to read CompactSize",
        ));
    }

    let prefix = transaction_bytes[0];
    *transaction_bytes = &transaction_bytes[1..];

    match prefix {
        0x00..=0xfc => Ok(prefix as u64),

        0xfd => {
            if transaction_bytes.len() < 2 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "not enough bytes for CompactSize u16",
                ));
            }

            let value = u16::from_le_bytes([transaction_bytes[0], transaction_bytes[1]]);

            *transaction_bytes = &transaction_bytes[2..];

            Ok(value as u64)
        }

        0xfe => {
            if transaction_bytes.len() < 4 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "not enough bytes for CompactSize u32",
                ));
            }

            let value = u32::from_le_bytes([
                transaction_bytes[0],
                transaction_bytes[1],
                transaction_bytes[2],
                transaction_bytes[3],
            ]);

            *transaction_bytes = &transaction_bytes[4..];

            Ok(value as u64)
        }

        0xff => {
            if transaction_bytes.len() < 8 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "not enough bytes for CompactSize u64",
                ));
            }

            let value = u64::from_le_bytes([
                transaction_bytes[0],
                transaction_bytes[1],
                transaction_bytes[2],
                transaction_bytes[3],
                transaction_bytes[4],
                transaction_bytes[5],
                transaction_bytes[6],
                transaction_bytes[7],
            ]);

            *transaction_bytes = &transaction_bytes[8..];

            Ok(value)
        }
    }
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    if transaction_bytes.len() < 32 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "not enough bytes to read TXID",
        ));
    }

    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&transaction_bytes[..32]);

    *transaction_bytes = &transaction_bytes[32..];

    Ok(Txid::from_bytes(bytes))
}

fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let size = read_compact_size(transaction_bytes)?;
    Ok(size.to_string())
}

fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let first_hash = Sha256::digest(row_transaction_bytes);
    let second_hash = Sha256::digest(first_hash);

    let mut txid_bytes = [0u8; 32];

    // TXIDs are conventionally displayed with the hash bytes reversed.
    txid_bytes.copy_from_slice(&second_hash);
    txid_bytes.reverse();

    Ok(Txid::from_bytes(txid_bytes))
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let raw_transaction = hex_to_bytes(&transaction_hex)?;
    let mut transaction_bytes: &[u8] = &raw_transaction;

    if transaction_bytes.len() < 4 {
        return Err(Error::new(ErrorKind::UnexpectedEof, "transaction is too short").into());
    }

    let version = read_version_byte(&mut transaction_bytes)?;

    // Detect SegWit marker + flag.
    let segwit = transaction_bytes.len() >= 2
        && transaction_bytes[0] == 0x00
        && transaction_bytes[1] != 0x00;

    if segwit {
        transaction_bytes = &transaction_bytes[2..];
    }

    // Inputs
    let input_count = read_compact_size(&mut transaction_bytes)?;

    let mut inputs = Vec::with_capacity(input_count as usize);

    for _ in 0..input_count {
        let txid = read_txid(&mut transaction_bytes)?;

        let output_index = read_u32(&mut transaction_bytes)?;

        let script_size = read_script_size(&mut transaction_bytes)?;

        let script_size: usize = script_size
            .parse()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid script size"))?;

        if transaction_bytes.len() < script_size {
            return Err(
                Error::new(ErrorKind::UnexpectedEof, "not enough bytes for scriptSig").into(),
            );
        }

        let script_sig = transaction_bytes[..script_size].to_vec();

        transaction_bytes = &transaction_bytes[script_size..];

        let sequence = read_u32(&mut transaction_bytes)?;

        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
        });
    }

    // Outputs
    let output_count = read_compact_size(&mut transaction_bytes)?;

    let mut outputs = Vec::with_capacity(output_count as usize);

    for _ in 0..output_count {
        let amount = read_amount(&mut transaction_bytes)?;

        let script_size = read_script_size(&mut transaction_bytes)?;

        let script_size: usize = script_size
            .parse()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid output script size"))?;

        if transaction_bytes.len() < script_size {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "not enough bytes for scriptPubKey",
            )
            .into());
        }

        let script_pubkey = transaction_bytes[..script_size].to_vec();

        transaction_bytes = &transaction_bytes[script_size..];

        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    // SegWit witness data comes after outputs and before locktime.
    // The current Transaction model doesn't store witness data,
    // so we parse it to advance the byte slice correctly.
    if segwit {
        for _ in 0..input_count {
            let witness_count = read_compact_size(&mut transaction_bytes)?;

            for _ in 0..witness_count {
                let witness_size = read_compact_size(&mut transaction_bytes)?;

                let witness_size = usize::try_from(witness_size)
                    .map_err(|_| Error::new(ErrorKind::InvalidData, "witness size is too large"))?;

                if transaction_bytes.len() < witness_size {
                    return Err(Error::new(
                        ErrorKind::UnexpectedEof,
                        "not enough bytes for witness data",
                    )
                    .into());
                }

                transaction_bytes = &transaction_bytes[witness_size..];
            }
        }
    }

    let lock_time = read_u32(&mut transaction_bytes)?;

    if !transaction_bytes.is_empty() {
        return Err(Error::new(ErrorKind::InvalidData, "unexpected bytes after locktime").into());
    }

    let transaction_id = hash_row_transaction(&raw_transaction)?;

    let transaction = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };

    Ok(serde_json::to_string_pretty(&transaction)?)
}
