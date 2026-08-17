use serde::{Serialize, Serializer};

#[derive(Debug, Serialize)]
pub struct Transaction {
    pub transaction_id: Txid,
    /// Only present for SegWit transactions (`None` for legacy transactions).
    /// wtxid = double-SHA256 of the *full* wire serialization, marker/flag/witness included.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wtxid: Option<Txid>,
    pub version: u32,
    pub is_segwit: bool,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub lock_time: u32,
}

#[derive(Debug, Serialize)]
pub struct Input {
    pub txid: Txid, // [u8; 32],
    pub output_index: u32,
    #[serde(serialize_with = "as_hex")]
    pub script_sig: Vec<u8>,
    pub sequence: u32,
    /// Empty for legacy inputs. One entry per witness stack item for SegWit inputs.
    #[serde(serialize_with = "as_hex_list")]
    pub witness: Vec<Vec<u8>>,
}

#[derive(Debug, Serialize)]
pub struct Output {
    #[serde(serialize_with = "as_btc")]
    pub amount: Amount,
    #[serde(serialize_with = "as_hex")]
    pub script_pubkey: Vec<u8>,
}

/// Serde helper: any `BitcoinValue` (currently just `Amount`) is written to JSON
/// as its BTC-denominated `f64`, not its raw satoshi integer.
fn as_btc<S: Serializer, T: BitcoinValue>(t: &T, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_f64(t.to_btc())
}

/// Serde helper: write raw bytes as a lowercase hex string instead of a JSON
/// array of numbers (which is what `Vec<u8>` would serialize to by default).
fn as_hex<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&hex::encode(bytes))
}

/// Same idea as `as_hex`, but for a witness stack: a `Vec` of byte blobs.
fn as_hex_list<S: Serializer>(items: &[Vec<u8>], s: S) -> Result<S::Ok, S::Error> {
    let hexed: Vec<String> = items.iter().map(hex::encode).collect();
    hexed.serialize(s)
}

/// A transaction amount, always stored internally as satoshis (the only unit
/// Bitcoin itself ever uses) so that arithmetic never touches floating point.
#[derive(Debug, Clone, Copy)]
pub struct Amount(u64);

impl Amount {
    // type associated functiion that initiate the instance of the struct i.e Amount
    pub fn from_sat(satoshi: u64) -> Amount {
        Amount(satoshi)
    }
}

// [u8; 32] => array of 32 element each element is 1 byte [u8]; i.e one byte is u8;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Txid([u8; 32]);

impl Txid {
    pub fn from_bytes(bytes: [u8; 32]) -> Txid {
        Txid(bytes)
    }

    /// Bitcoin displays txids byte-reversed relative to how they are hashed
    /// and stored on the wire. `hash_row_transaction` gives us the raw,
    /// internal-order hash; this flips it into the familiar block-explorer form.
    pub fn to_hex(self) -> String {
        let mut reversed = self.0;
        reversed.reverse();
        hex::encode(reversed)
    }
}

impl Serialize for Txid {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_hex())
    }
}

trait BitcoinValue {
    fn to_btc(&self) -> f64;
}

impl BitcoinValue for Amount {
    fn to_btc(&self) -> f64 {
        self.0 as f64 / 100_000_000.0
    }
}
