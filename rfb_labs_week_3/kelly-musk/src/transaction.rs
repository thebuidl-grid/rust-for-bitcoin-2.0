//! The in-memory transaction model plus the byte-level parser and serializer.
//!
//! Nothing here uses a Bitcoin library: the wire format is implemented directly
//! from BIP144 (SegWit serialization) and the classic pre-SegWit layout.

use sha2::{Digest, Sha256};

use crate::error::{ParseError, Result};
use crate::reader::{write_compact_size, ByteReader};

/// A reference to a previous transaction output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutPoint {
    /// 32-byte txid in internal (little-endian) byte order, as it appears on the wire.
    pub txid: [u8; 32],
    pub vout: u32,
}

impl OutPoint {
    /// The all-zero txid with vout `0xffffffff` marks a coinbase input.
    pub fn is_coinbase(&self) -> bool {
        self.txid == [0_u8; 32] && self.vout == 0xffff_ffff
    }

    /// txid in the reversed order used for display and RPC output.
    pub fn txid_hex(&self) -> String {
        hash_to_display_hex(&self.txid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxIn {
    pub previous_output: OutPoint,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
    /// One entry per witness stack item; empty for legacy inputs.
    pub witness: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxOut {
    /// Amount in satoshis.
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub version: i32,
    /// Whether the SegWit marker+flag were present on the wire.
    pub segwit: bool,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub lock_time: u32,
}

impl Transaction {
    /// Parse a full transaction from raw bytes, rejecting trailing data.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        let mut reader = ByteReader::new(bytes);
        let tx = Self::parse_from(&mut reader)?;
        if reader.remaining() != 0 {
            return Err(ParseError::TrailingBytes {
                consumed: reader.position(),
                total: reader.total(),
            });
        }
        Ok(tx)
    }

    fn parse_from(reader: &mut ByteReader<'_>) -> Result<Self> {
        let version = reader.read_u32_le("version")? as i32;

        // BIP144: a `0x00` where the input count belongs is the SegWit marker.
        let mut segwit = false;
        let input_count = {
            let marker = reader.read_compact_size("input count / SegWit marker")?;
            if marker == 0 {
                let flag = reader.read_u8("SegWit flag")?;
                if flag != 0x01 {
                    return Err(ParseError::BadSegwitFlag {
                        offset: reader.position() - 1,
                        flag,
                    });
                }
                segwit = true;
                let real = reader.read_compact_size("input count")?;
                if real == 0 {
                    return Err(ParseError::EmptySegwitInputs);
                }
                real
            } else {
                marker
            }
        };

        let mut inputs = Vec::with_capacity(input_count.min(4096) as usize);
        for _ in 0..input_count {
            let txid = reader.read_array::<32>("input txid")?;
            let vout = reader.read_u32_le("input vout")?;
            let script_len = reader.read_compact_size("scriptSig length")?;
            let script_sig = reader
                .read_bytes(script_len as usize, "scriptSig")?
                .to_vec();
            let sequence = reader.read_u32_le("input sequence")?;
            inputs.push(TxIn {
                previous_output: OutPoint { txid, vout },
                script_sig,
                sequence,
                witness: Vec::new(),
            });
        }

        let output_count = reader.read_compact_size("output count")?;
        let mut outputs = Vec::with_capacity(output_count.min(4096) as usize);
        for _ in 0..output_count {
            let value = reader.read_u64_le("output value")?;
            let script_len = reader.read_compact_size("scriptPubKey length")?;
            let script_pubkey = reader
                .read_bytes(script_len as usize, "scriptPubKey")?
                .to_vec();
            outputs.push(TxOut {
                value,
                script_pubkey,
            });
        }

        if segwit {
            for input in &mut inputs {
                let items = reader.read_compact_size("witness item count")?;
                let mut stack = Vec::with_capacity(items.min(4096) as usize);
                for _ in 0..items {
                    let len = reader.read_compact_size("witness item length")?;
                    stack.push(reader.read_bytes(len as usize, "witness item")?.to_vec());
                }
                input.witness = stack;
            }
        }

        let lock_time = reader.read_u32_le("locktime")?;

        Ok(Self {
            version,
            segwit,
            inputs,
            outputs,
            lock_time,
        })
    }

    /// Does any input carry witness data?
    pub fn has_witness(&self) -> bool {
        self.inputs.iter().any(|input| !input.witness.is_empty())
    }

    /// Serialize without the marker/flag/witness (pre-SegWit layout). This is the
    /// preimage for the txid.
    pub fn serialize_legacy(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(self.version as u32).to_le_bytes());
        write_compact_size(&mut out, self.inputs.len() as u64);
        for input in &self.inputs {
            out.extend_from_slice(&input.previous_output.txid);
            out.extend_from_slice(&input.previous_output.vout.to_le_bytes());
            write_compact_size(&mut out, input.script_sig.len() as u64);
            out.extend_from_slice(&input.script_sig);
            out.extend_from_slice(&input.sequence.to_le_bytes());
        }
        write_compact_size(&mut out, self.outputs.len() as u64);
        for output in &self.outputs {
            out.extend_from_slice(&output.value.to_le_bytes());
            write_compact_size(&mut out, output.script_pubkey.len() as u64);
            out.extend_from_slice(&output.script_pubkey);
        }
        out.extend_from_slice(&self.lock_time.to_le_bytes());
        out
    }

    /// Serialize exactly as parsed: with marker/flag/witness when `segwit` is set.
    pub fn serialize(&self) -> Vec<u8> {
        if !self.segwit {
            return self.serialize_legacy();
        }
        let mut out = Vec::new();
        out.extend_from_slice(&(self.version as u32).to_le_bytes());
        out.push(0x00); // marker
        out.push(0x01); // flag
        write_compact_size(&mut out, self.inputs.len() as u64);
        for input in &self.inputs {
            out.extend_from_slice(&input.previous_output.txid);
            out.extend_from_slice(&input.previous_output.vout.to_le_bytes());
            write_compact_size(&mut out, input.script_sig.len() as u64);
            out.extend_from_slice(&input.script_sig);
            out.extend_from_slice(&input.sequence.to_le_bytes());
        }
        write_compact_size(&mut out, self.outputs.len() as u64);
        for output in &self.outputs {
            out.extend_from_slice(&output.value.to_le_bytes());
            write_compact_size(&mut out, output.script_pubkey.len() as u64);
            out.extend_from_slice(&output.script_pubkey);
        }
        for input in &self.inputs {
            write_compact_size(&mut out, input.witness.len() as u64);
            for item in &input.witness {
                write_compact_size(&mut out, item.len() as u64);
                out.extend_from_slice(item);
            }
        }
        out.extend_from_slice(&self.lock_time.to_le_bytes());
        out
    }

    /// Byte length of the SegWit-stripped serialization.
    pub fn base_size(&self) -> usize {
        self.serialize_legacy().len()
    }

    /// Byte length of the full serialization (== Core's `size`).
    pub fn total_size(&self) -> usize {
        self.serialize().len()
    }

    /// BIP141 weight: `base_size * 3 + total_size`.
    pub fn weight(&self) -> usize {
        self.base_size() * 3 + self.total_size()
    }

    /// BIP141 virtual size: `ceil(weight / 4)`.
    pub fn vsize(&self) -> usize {
        self.weight().div_ceil(4)
    }

    /// Double-SHA256 of the legacy serialization, reversed for display.
    pub fn txid(&self) -> String {
        hash_to_display_hex(&double_sha256(&self.serialize_legacy()))
    }

    /// Double-SHA256 of the full (witness) serialization, reversed for display.
    /// Equal to the txid for non-SegWit transactions.
    pub fn wtxid(&self) -> String {
        hash_to_display_hex(&double_sha256(&self.serialize()))
    }
}

pub fn double_sha256(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(first);
    second.into()
}

/// Bitcoin displays 32-byte hashes in reverse byte order.
fn hash_to_display_hex(hash: &[u8; 32]) -> String {
    let mut reversed = *hash;
    reversed.reverse();
    hex::encode(reversed)
}
