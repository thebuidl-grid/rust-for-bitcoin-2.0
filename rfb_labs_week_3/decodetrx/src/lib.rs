use std::io::{Error, ErrorKind, Read};
// use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256}; // https://docs.rs/sha2/latest/sha2/
use transaction::{Amount, Input, Output, Transaction, Txid};
mod transaction;

// #[derive(Parser)]
// #[command(name= " Transaction decoder")]
// #[command(version= "1.0")]
// #[command(about= "Bitcoin Transaction decoder", long_about=None)]
// struct CLI {
//       #[arg(
//             required = true,
//             help="(string, required) Row Transaction hex"
//         )]
//     transaction_hex: String
// }

// The version is the first field of every transaction: 4 bytes, little-endian.
// This variant works straight off the hex string; once we are walking the
// transaction with a cursor we use `read_version_byte` instead.
#[allow(dead_code)]
fn read_version(transaction_hex: &str) -> u32 {
    let transaction_bytes = hex::decode(transaction_hex).unwrap();
    let mut bytes_slice = transaction_bytes.as_slice();
    read_version_byte(&mut bytes_slice).unwrap()
}

fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    let mut buffer = [0_u8; 8];
    // Reading from a `&[u8]` advances the slice itself, so the caller's cursor
    // moves forward by the number of bytes we consumed.
    let _ = transaction_bytes.read(&mut buffer);
    u64::from_le_bytes(buffer)
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    if transaction_bytes.len() < 8 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "not enough bytes for an amount",
        ));
    }
    Ok(Amount::from_sat(read_u64(transaction_bytes)))
}

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    let mut buffer = [0_u8; 4];
    bytes_slice.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    // CompactSize: one marker byte decides how many more bytes carry the value.
    let mut marker = [0_u8; 1];
    transaction_bytes.read_exact(&mut marker)?;

    match marker[0] {
        0x00..=0xFC => Ok(marker[0] as u64),
        0xFD => {
            let mut buffer = [0_u8; 2];
            transaction_bytes.read_exact(&mut buffer)?;
            Ok(u16::from_le_bytes(buffer) as u64)
        }
        0xFE => Ok(read_u32(transaction_bytes)? as u64),
        0xFF => {
            if transaction_bytes.len() < 8 {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "not enough bytes for a compact size",
                ));
            }
            Ok(read_u64(transaction_bytes))
        }
    }
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    let mut buffer = [0_u8; 32];
    transaction_bytes.read_exact(&mut buffer)?;
    Ok(Txid::from_bytes(buffer))
}

fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    // A script is length-prefixed with a CompactSize, then that many raw bytes.
    let script_size = read_compact_size(transaction_bytes)? as usize;
    if transaction_bytes.len() < script_size {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "script is shorter than its length prefix",
        ));
    }

    let mut buffer = vec![0_u8; script_size];
    transaction_bytes.read_exact(&mut buffer)?;
    Ok(hex::encode(buffer))
}

fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}
// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    // A txid is SHA256 applied twice over the legacy serialisation.
    let mut hasher = Sha256::new();
    hasher.update(row_transaction_bytes);
    let first_hash = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(first_hash);
    let second_hash = hasher.finalize();

    Ok(Txid::from_bytes(second_hash.into()))
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let transaction_bytes = hex::decode(transaction_hex)?;
    let mut bytes_slice = transaction_bytes.as_slice();

    let version = read_version_byte(&mut bytes_slice)?;

    // A 0x00 where the input count belongs is the SegWit marker (BIP 144). It is
    // followed by a flag byte, and only then by the real input count.
    let mut input_count = read_compact_size(&mut bytes_slice)?;
    let is_segwit = input_count == 0;
    if is_segwit {
        let mut flag = [0_u8; 1];
        bytes_slice.read_exact(&mut flag)?;
        if flag[0] != 0x01 {
            return Err(Box::new(Error::new(
                ErrorKind::InvalidData,
                format!("unsupported segwit flag byte {:#04x}", flag[0]),
            )));
        }
        input_count = read_compact_size(&mut bytes_slice)?;
    }

    let mut inputs = Vec::new();
    for _ in 0..input_count {
        let txid = read_txid(&mut bytes_slice)?;
        let output_index = read_u32(&mut bytes_slice)?;
        let script_sig = read_script_size(&mut bytes_slice)?;
        let sequence = read_u32(&mut bytes_slice)?;

        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
            witness: None,
        });
    }

    let output_count = read_compact_size(&mut bytes_slice)?;
    let mut outputs = Vec::new();
    for _ in 0..output_count {
        let amount = read_amount(&mut bytes_slice)?;
        let script_pubkey = read_script_size(&mut bytes_slice)?;

        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    // Where the witness section starts, i.e. the first byte the txid does not commit to.
    let witness_start = transaction_bytes.len() - bytes_slice.len();

    if is_segwit {
        // Every input carries its own witness stack, in input order. Each item
        // is a length-prefixed blob, the same shape as a script, so the same
        // reader handles it.
        for input in inputs.iter_mut() {
            let item_count = read_compact_size(&mut bytes_slice)?;
            let mut items = Vec::new();
            for _ in 0..item_count {
                items.push(read_script_size(&mut bytes_slice)?);
            }
            input.witness = Some(items);
        }
    }

    let lock_time = read_u32(&mut bytes_slice)?;

    if !bytes_slice.is_empty() {
        return Err(Box::new(Error::new(
            ErrorKind::InvalidData,
            format!(
                "{} trailing byte(s) after the transaction",
                bytes_slice.len()
            ),
        )));
    }

    // The txid always hashes the legacy serialisation, so for a SegWit
    // transaction we stitch it back together without the marker, flag and
    // witnesses: version, inputs, outputs, lock time.
    let end = transaction_bytes.len() - bytes_slice.len();
    let transaction_id = if is_segwit {
        let mut legacy_bytes = Vec::with_capacity(witness_start - 2 + 4);
        legacy_bytes.extend_from_slice(&transaction_bytes[..4]);
        legacy_bytes.extend_from_slice(&transaction_bytes[6..witness_start]);
        legacy_bytes.extend_from_slice(&transaction_bytes[end - 4..end]);
        hash_row_transaction(&legacy_bytes)?
    } else {
        hash_row_transaction(&transaction_bytes[..end])?
    };

    let transaction = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };

    Ok(serde_json::to_string_pretty(&transaction)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The same raw transaction the trxparse demo walks through: a v2 P2WPKH spend
    // with one input, two outputs and a single witness stack.
    const SEGWIT_TX: &str = "0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000";

    // The first ever spend: block 170, Satoshi to Hal Finney.
    const LEGACY_TX: &str = "0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000";

    #[test]
    fn reads_the_version() {
        assert_eq!(read_version(SEGWIT_TX), 2);
        assert_eq!(read_version(LEGACY_TX), 1);
    }

    #[test]
    fn reads_every_compact_size_encoding() {
        assert_eq!(read_compact_size(&mut [0xfc_u8].as_slice()).unwrap(), 252);
        assert_eq!(
            read_compact_size(&mut [0xfd, 0x00, 0x01].as_slice()).unwrap(),
            256
        );
        assert_eq!(
            read_compact_size(&mut [0xfe, 0x00, 0x00, 0x01, 0x00].as_slice()).unwrap(),
            65_536
        );
        assert_eq!(
            read_compact_size(&mut [0xff, 0, 0, 0, 0, 0x01, 0, 0, 0].as_slice()).unwrap(),
            4_294_967_296
        );
    }

    #[test]
    fn truncated_input_is_an_error() {
        assert!(read_u32(&mut [0x01, 0x02].as_slice()).is_err());
        assert!(read_amount(&mut [0x01, 0x02].as_slice()).is_err());
        assert!(decode_transaction(SEGWIT_TX[..40].to_string()).is_err());
    }

    #[test]
    fn decodes_a_segwit_transaction() {
        let decoded: serde_json::Value =
            serde_json::from_str(&decode_transaction(SEGWIT_TX.to_string()).unwrap()).unwrap();

        assert_eq!(
            decoded["transaction_id"],
            "be9ea29072566edbc6827e3d9caf1d8c0b57cb0d5e74b95c721c46b3124cbe0b"
        );
        assert_eq!(decoded["version"], 2);
        assert_eq!(decoded["lock_time"], 0);

        assert_eq!(decoded["inputs"].as_array().unwrap().len(), 1);
        assert_eq!(
            decoded["inputs"][0]["txid"],
            "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796"
        );
        assert_eq!(decoded["inputs"][0]["output_index"], 1);
        // A native SegWit spend has an empty script_sig; the signature is in the witness.
        assert_eq!(decoded["inputs"][0]["script_sig"], "");
        assert_eq!(decoded["inputs"][0]["sequence"], 4_294_967_295_u32);

        // A P2WPKH witness is a two item stack: signature then public key.
        let witness = decoded["inputs"][0]["witness"].as_array().unwrap();
        assert_eq!(witness.len(), 2);
        assert_eq!(
            witness[0],
            "304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c201"
        );
        assert_eq!(
            witness[1],
            "020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f1"
        );

        assert_eq!(decoded["outputs"].as_array().unwrap().len(), 2);
        assert_eq!(decoded["outputs"][0]["amount"], 0.000001);
        assert_eq!(
            decoded["outputs"][0]["script_pubkey"],
            "0014274ae586ad2035efb4c25049c155f98310d7e106"
        );
        assert_eq!(decoded["outputs"][1]["amount"], 0.04462282);
    }

    // Hand built rather than taken from a block, to reach shapes the two real
    // samples do not: several inputs, a witness stack that is empty on one input
    // while the others carry items, and an amount spanning the full u64 range.
    #[test]
    fn decodes_multiple_inputs_and_mixed_witness_stacks() {
        const TX: &str = "02000000000103000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f0100000000fdffffff202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3ffe00000000fdffffffabababababababababababababababababababababababababababababababab0700000048471111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111fdffffff02a086010000000000160014cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd0040075af0750700016a0248303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030210202020202020202020202020202020202020202020202020202020202020202020140515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151510020a10700";

        let decoded: serde_json::Value =
            serde_json::from_str(&decode_transaction(TX.to_string()).unwrap()).unwrap();

        assert_eq!(
            decoded["transaction_id"],
            "6242bb2516c7ea43496586d8351c7e302e18b1ed1e90c15babf3836296ce8166"
        );
        assert_eq!(decoded["lock_time"], 500_000);

        let inputs = decoded["inputs"].as_array().unwrap();
        assert_eq!(inputs.len(), 3);
        assert_eq!(inputs[0]["witness"].as_array().unwrap().len(), 2);
        assert_eq!(inputs[1]["witness"].as_array().unwrap().len(), 1);
        // An input may contribute no witness items at all, and its empty stack
        // still has to be consumed to stay aligned with the next one.
        assert_eq!(inputs[2]["witness"].as_array().unwrap().len(), 0);
        assert_eq!(inputs[2]["script_sig"].as_str().unwrap().len() / 2, 72);

        // 2_100_000_000_000_000 sats, the whole supply, well past u32.
        assert_eq!(decoded["outputs"][1]["amount"], 21_000_000.0);
    }

    #[test]
    fn decodes_a_legacy_transaction() {
        let decoded: serde_json::Value =
            serde_json::from_str(&decode_transaction(LEGACY_TX.to_string()).unwrap()).unwrap();

        assert_eq!(
            decoded["transaction_id"],
            "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16"
        );
        assert_eq!(decoded["version"], 1);
        assert_eq!(decoded["inputs"].as_array().unwrap().len(), 1);
        // No witness section on a legacy transaction, so the key is absent.
        assert!(decoded["inputs"][0].get("witness").is_none());
        assert_eq!(decoded["outputs"].as_array().unwrap().len(), 2);
        assert_eq!(decoded["outputs"][0]["amount"], 10.0);
        assert_eq!(decoded["outputs"][1]["amount"], 40.0);
    }
}
