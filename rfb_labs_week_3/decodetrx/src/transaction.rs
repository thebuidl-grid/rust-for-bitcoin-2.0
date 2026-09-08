use serde::{Serialize, Serializer};

// ──────────────────────────────────────────────
// Top-level transaction struct
// ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct Transaction {
    pub transaction_id: Txid,
    pub version: u32,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub lock_time: u32,
}

// ──────────────────────────────────────────────
// Input: one spending reference to a previous output
// ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct Input {
    // The previous transaction's ID (32 bytes, stored as hex)
    pub txid: Txid,
    // Which output of that previous transaction we are spending
    pub output_index: u32,
    // Unlocking script (empty for native SegWit inputs)
    pub script_sig: String,
    // Relative lock-time / RBF signal
    pub sequence: u32,
}

// ──────────────────────────────────────────────
// Output: one payment destination
// ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct Output {
    // Serialize satoshis as a BTC decimal string via the custom serializer below
    #[serde(serialize_with = "as_btc")]
    pub amount: Amount,
    // Locking script expressed as a hex string
    pub script_pubkey: String,
}

// ──────────────────────────────────────────────
// Amount: wraps satoshis (u64)
// ──────────────────────────────────────────────

#[derive(Debug)]
pub struct Amount(pub u64);

impl Amount {
    /// Create an Amount from a satoshi value.
    pub fn from_sat(satoshi: u64) -> Amount {
        Amount(satoshi)
    }
}

// ──────────────────────────────────────────────
// BitcoinValue trait: anything that can express itself in BTC
// ──────────────────────────────────────────────

pub trait BitcoinValue {
    fn to_btc(&self) -> f64;
}

impl BitcoinValue for Amount {
    /// Convert satoshis to BTC.
    /// 1 BTC = 100,000,000 satoshis  →  divide by 1e8
    fn to_btc(&self) -> f64 {
        // self.0 is the inner u64 satoshi value
        self.0 as f64 / 100_000_000.0
    }
}

// ──────────────────────────────────────────────
// Custom serde serializer: writes Amount as a BTC f64
// ──────────────────────────────────────────────

/// Used by the `#[serde(serialize_with = "as_btc")]` attribute on Output::amount.
/// Converts any BitcoinValue to its BTC representation and serializes it as an f64.
pub fn as_btc<S: Serializer, T: BitcoinValue>(t: &T, s: S) -> Result<S::Ok, S::Error> {
    // Call to_btc() which divides satoshis by 1e8, then hand the f64 to the serializer
    s.serialize_f64(t.to_btc())
}

// ──────────────────────────────────────────────
// Txid: a 32-byte transaction hash
// ──────────────────────────────────────────────

#[derive(Debug)]
pub struct Txid(pub [u8; 32]);

impl Txid {
    /// Build a Txid from a raw 32-byte array.
    pub fn from_bytes(bytes: [u8; 32]) -> Txid {
        Txid(bytes)
    }
}

impl Serialize for Txid {
    /// Bitcoin displays TXIDs in **reversed** byte order (big-endian display of a
    /// little-endian hash).  We reverse the bytes and then hex-encode them so the
    /// output matches what block explorers show.
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        // Reverse the 32-byte array: Bitcoin stores hashes little-endian but
        // displays them big-endian.
        let mut reversed = self.0;
        reversed.reverse();
        // hex-encode to a lowercase hex string, e.g. "a1b2c3..."
        s.serialize_str(&hex::encode(reversed))
    }
}
