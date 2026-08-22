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
    #[serde(serialize_with = "as_hex")]
    pub script_sig: Vec<u8>,
    pub sequence: u32,
}

#[derive(Debug, Serialize)]
pub struct Output {
    #[serde(serialize_with = "as_btc")]
    pub amount: Amount,
    #[serde(serialize_with = "as_hex")]
    pub script_pubkey: Vec<u8>,
}

fn as_btc<S: Serializer, T: BitcoinValue>(t: &T, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_f64(t.to_btc())
}

fn as_hex<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&hex::encode(bytes))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amount(pub u64);

impl Amount {
    pub fn from_sat(satoshi: u64) -> Amount {
        Amount(satoshi)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Txid(pub [u8; 32]);

impl Txid {
    pub fn from_bytes(bytes: [u8; 32]) -> Txid {
        Txid(bytes)
    }

    pub fn to_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl Serialize for Txid {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut bytes = self.0;
        bytes.reverse();
        s.serialize_str(&hex::encode(bytes))
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
