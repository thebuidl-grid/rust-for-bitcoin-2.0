
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
    pub txid: Txid, // [u8; 32],
    pub output_index: u32,
    #[serde(serialize_with = "as_hex")]
    pub script_sig: Vec<u8>,
    #[serde(serialize_with = "sequence_as_hex")]
    pub sequence: u32,
    #[serde(serialize_with = "witness_as_hex")]
    pub witness: Vec<Vec<u8>>,
}

#[derive(Debug, Serialize)]
pub struct Output {
    #[serde(serialize_with = "as_sats")]
    pub amount: Amount,
    #[serde(serialize_with = "as_hex")]
    pub script_pubkey: Vec<u8>,
}

fn as_sats<S: Serializer>(amount: &Amount, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_u64(amount.0)
}

fn as_hex<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&hex::encode(bytes))
}

fn witness_as_hex<S: Serializer>(items: &[Vec<u8>], s: S) -> Result<S::Ok, S::Error> {
    s.collect_seq(items.iter().map(hex::encode))
}

fn sequence_as_hex<S: Serializer>(sequence: &u32, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&format!("{:08x}", sequence))
}

#[derive(Debug)]
pub struct Amount( u64);

impl Amount {
  // type associated functiion that initiate the instance of the struct i.e Amount
  pub fn from_sat(satoshi: u64) -> Amount {
    Amount(satoshi)
  }
}

#[derive(Debug)]
pub struct Txid([u8; 32]);

// [u8; 32] => array of 32 element each element is 1 byte [u8]; i.e one byte is u8;

impl Txid {
    pub fn from_bytes(bytes: [u8; 32]) -> Txid {
        Txid(bytes)
    }
}

impl Serialize for Txid {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(self.0))
    }
}

#[allow(dead_code)]
trait BitcoinValue {
    fn to_btc(&self) -> f64;
}

#[allow(dead_code)]
impl BitcoinValue for Amount {
    fn to_btc(&self) -> f64 {
        self.0 as f64 / 100_000_000.0
    }
}


