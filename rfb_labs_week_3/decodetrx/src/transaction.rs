use serde::{Serialize, Serializer};

#[derive(Debug, Serialize)]
pub struct Transaction {
    pub transaction_id: Txid,
    pub version: u32,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub lock_time: u32,
}

#[derive(Debug, Serialize)]
pub struct Input {
    pub txid: Txid,
    pub output_index: u32,
    pub script_sig: String,
    pub sequence: u32,
    /// Witness stack for this input, as hex strings. Empty for legacy inputs,
    /// and omitted from the JSON entirely so legacy output stays clean.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub witness: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Output {
    #[serde(serialize_with = "as_btc")]
    pub amount: Amount,
    pub script_pubkey: String,
}

// Emitting the f64 directly would print small amounts in scientific notation
// (100 sats becomes `1e-6`). Bitcoin amounts always carry 8 decimal places, so
// format to a fixed 8 and hand serde the digits as a raw JSON number — that
// keeps it an unquoted number, matching what `bitcoin-cli` prints.
fn as_btc<S: Serializer, T: BitcoinValue>(t: &T, s: S) -> Result<S::Ok, S::Error> {
    let formatted = format!("{:.8}", t.to_btc());
    let number = serde_json::value::RawValue::from_string(formatted)
        .map_err(serde::ser::Error::custom)?;
    number.serialize(s)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amount(u64);

impl Amount {
    // type associated functiion that initiate the instance of the struct i.e Amount
    pub fn from_sat(satoshi: u64) -> Amount {
        Amount(satoshi)
    }

    pub fn to_sat(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Txid([u8; 32]);

// [u8; 32] => array of 32 element each element is 1 byte [u8]; i.e one byte is u8;

impl Txid {
    /// Takes the 32 bytes exactly as they appear on the wire (internal byte
    /// order). Display reverses them — see the `Serialize` impl below.
    pub fn from_bytes(bytes: [u8; 32]) -> Txid {
        Txid(bytes)
    }
}

impl Serialize for Txid {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl std::fmt::Display for Txid {
    // Bitcoin stores txids in internal byte order on the wire, but every
    // explorer and RPC displays them reversed (big-endian). Reverse on the way
    // out so the value matches what mempool.space shows.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0.iter().rev() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

pub trait BitcoinValue {
    fn to_btc(&self) -> f64;
}

impl BitcoinValue for Amount {
    fn to_btc(&self) -> f64 {
        self.0 as f64 / 100_000_000.0
    }
}
