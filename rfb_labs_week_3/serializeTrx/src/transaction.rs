//! The transaction model and the serialiser.
//!
//! The serialisation is the one from the original program, unchanged: same
//! field order, same little-endian integers, same CompactSize prefixes. What
//! changed is where the values come from. See [`crate::cli`].
//!
//! ```text
//! ┌──────────────────────────────┐
//! │ Version          4 bytes     │
//! ├──────────────────────────────┤
//! │ Marker           1 byte      │  SegWit only (0x00)
//! │ Flag             1 byte      │  SegWit only (0x01)
//! ├──────────────────────────────┤
//! │ Input count      VarInt      │
//! │ Inputs           Variable    │
//! ├──────────────────────────────┤
//! │ Output count     VarInt      │
//! │ Outputs          Variable    │
//! ├──────────────────────────────┤
//! │ Witness          Variable    │  SegWit only
//! ├──────────────────────────────┤
//! │ Locktime         4 bytes     │
//! └──────────────────────────────┘
//! ```

/// One spent output, together with the witness stack that unlocks it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxInput {
    /// The previous txid in internal byte order, i.e. ready to be written out
    /// as is. The command line takes the reversed, explorer facing order by
    /// default and flips it before it gets here.
    pub prev_txid: Vec<u8>,
    pub vout: u32,
    /// Empty for a native SegWit input: its signature lives in `witness`.
    pub script_sig: Vec<u8>,
    pub sequence: u32,
    /// The stack for this input. Empty is legal even in a SegWit transaction,
    /// and still costs one `0x00` byte in the witness section.
    pub witness: Vec<Vec<u8>>,
}

/// One newly created output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxOutput {
    /// Satoshis, not BTC.
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub locktime: u32,
    /// Whether to write the BIP144 marker, flag and witness section.
    pub segwit: bool,
}

impl Transaction {
    /// The full serialisation, witness included when the transaction is SegWit.
    pub fn serialize(&self) -> Vec<u8> {
        serialize_transaction(self)
    }

    /// The size in bytes of what [`Transaction::serialize`] produces.
    pub fn total_size(&self) -> usize {
        self.serialize().len()
    }

    /// The size of the legacy serialisation, i.e. the bytes a txid commits to.
    /// For a non-SegWit transaction this is the whole thing.
    pub fn base_size(&self) -> usize {
        serialize(self, false).len()
    }

    /// Weight units, as defined by BIP141: witness bytes count once, everything
    /// else counts four times.
    pub fn weight(&self) -> usize {
        self.base_size() * 3 + self.total_size()
    }

    /// Virtual size in vbytes: weight rounded up to the next whole unit.
    pub fn vsize(&self) -> usize {
        self.weight().div_ceil(4)
    }

    /// Total satoshis paid out. Fees are the difference against the inputs,
    /// which a serialiser cannot see, so only the output side is available.
    pub fn total_output_value(&self) -> u64 {
        self.outputs.iter().map(|output| output.value).sum()
    }
}

/// Serialise `trx`, writing the witness section when it is marked SegWit.
pub fn serialize_transaction(trx: &Transaction) -> Vec<u8> {
    serialize(trx, trx.segwit)
}

/// The serialiser proper. `with_witness` is what the original read straight
/// off `trx.segwit`; it is a parameter only so the legacy size can be measured
/// with the very same code rather than a second, drifting copy of it.
fn serialize(trx: &Transaction, with_witness: bool) -> Vec<u8> {
    let mut result = Vec::new();

    // VERSION
    // to_le_bytes converts the integer to its little-endian byte
    // representation; extend_from_slice appends those bytes to the result.
    result.extend_from_slice(&trx.version.to_le_bytes());

    // MARKER + FLAG
    // A 0x00 where the input count belongs cannot be a real count, which is
    // how a parser tells the two formats apart (BIP144).
    if with_witness {
        result.push(0x00); // marker
        result.push(0x01); // flag
    }

    // INPUT COUNT
    result.extend_from_slice(&encode_varint(trx.inputs.len()));

    // INPUTS
    for input in &trx.inputs {
        // Previous transaction ID
        result.extend_from_slice(&input.prev_txid);

        // Previous output index
        result.extend_from_slice(&input.vout.to_le_bytes());

        // ScriptSig length
        result.extend_from_slice(&encode_varint(input.script_sig.len()));

        // ScriptSig
        result.extend_from_slice(&input.script_sig);

        // Sequence
        result.extend_from_slice(&input.sequence.to_le_bytes());
    }

    // OUTPUT COUNT
    result.extend_from_slice(&encode_varint(trx.outputs.len()));

    // OUTPUTS
    for output in &trx.outputs {
        // Value in satoshis
        result.extend_from_slice(&output.value.to_le_bytes());

        // ScriptPubKey length
        result.extend_from_slice(&encode_varint(output.script_pubkey.len()));

        // ScriptPubKey
        result.extend_from_slice(&output.script_pubkey);
    }

    // WITNESSES
    // One stack per input, in input order, so an input with nothing to say
    // still writes its zero count to keep the stacks aligned.
    if with_witness {
        for input in &trx.inputs {
            // Number of witness items
            result.extend_from_slice(&encode_varint(input.witness.len()));

            for item in &input.witness {
                // Witness item length
                result.extend_from_slice(&encode_varint(item.len()));

                // Witness item
                result.extend_from_slice(item);
            }
        }
    }

    // LOCKTIME
    result.extend_from_slice(&trx.locktime.to_le_bytes());

    result
}

/// Bitcoin's CompactSize, used for every count and every length prefix:
/// number of inputs, number of outputs, script length, number of witness
/// items, witness item length.
///
/// ```text
/// 0 - 252                  store directly        [XX]
/// 253 - 65,535             FD + 2 bytes          [FD][XX XX]
/// 65,536 - 4,294,967,295   FE + 4 bytes          [FE][XX XX XX XX]
/// larger                   FF + 8 bytes          [FF][XX XX XX XX XX XX XX XX]
/// ```
pub fn encode_varint(value: usize) -> Vec<u8> {
    match value {
        0..=0xfc => vec![value as u8],

        0xfd..=0xffff => {
            let mut result = vec![0xfd];
            result.extend_from_slice(&(value as u16).to_le_bytes());
            result
        }

        0x10000..=0xffff_ffff => {
            let mut result = vec![0xfe];
            result.extend_from_slice(&(value as u32).to_le_bytes());
            result
        }

        _ => {
            let mut result = vec![0xff];
            result.extend_from_slice(&(value as u64).to_le_bytes());
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_size_switches_at_every_boundary() {
        assert_eq!(encode_varint(0), vec![0x00]);
        assert_eq!(encode_varint(252), vec![0xfc]);
        assert_eq!(encode_varint(253), vec![0xfd, 0xfd, 0x00]);
        assert_eq!(encode_varint(65_535), vec![0xfd, 0xff, 0xff]);
        assert_eq!(encode_varint(65_536), vec![0xfe, 0x00, 0x00, 0x01, 0x00]);
        assert_eq!(
            encode_varint(4_294_967_295),
            vec![0xfe, 0xff, 0xff, 0xff, 0xff]
        );
        assert_eq!(
            encode_varint(4_294_967_296),
            vec![0xff, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]
        );
    }

    fn one_input_transaction(segwit: bool, witness: Vec<Vec<u8>>) -> Transaction {
        Transaction {
            version: 2,
            inputs: vec![TxInput {
                prev_txid: vec![0x11; 32],
                vout: 0,
                script_sig: vec![],
                sequence: 0xffff_ffff,
                witness,
            }],
            outputs: vec![TxOutput {
                value: 1_000,
                script_pubkey: vec![0x51],
            }],
            locktime: 0,
            segwit,
        }
    }

    #[test]
    fn the_marker_and_flag_only_appear_on_segwit() {
        let legacy = one_input_transaction(false, vec![]).serialize();
        assert_eq!(&legacy[4..5], &[0x01]); // straight to the input count

        let segwit = one_input_transaction(true, vec![vec![0xaa]]).serialize();
        assert_eq!(&segwit[4..7], &[0x00, 0x01, 0x01]); // marker, flag, count
    }

    #[test]
    fn an_empty_witness_stack_still_costs_a_byte() {
        let with_stack = one_input_transaction(true, vec![vec![0xaa]]);
        let without = one_input_transaction(true, vec![]);
        // 0x00 count, versus 0x01 count + 0x01 length + the item itself.
        assert_eq!(with_stack.total_size() - without.total_size(), 2);
        assert_eq!(without.serialize().iter().rev().nth(4), Some(&0x00));
    }

    #[test]
    fn weight_counts_witness_bytes_once() {
        let trx = one_input_transaction(true, vec![vec![0xaa; 72]]);
        let witness_bytes = trx.total_size() - trx.base_size();
        assert_eq!(trx.weight(), trx.base_size() * 4 + witness_bytes);
        assert_eq!(trx.vsize(), trx.weight().div_ceil(4));

        // With no witness section at all the two sizes agree and vsize is size.
        let legacy = one_input_transaction(false, vec![]);
        assert_eq!(legacy.base_size(), legacy.total_size());
        assert_eq!(legacy.vsize(), legacy.total_size());
    }
}
