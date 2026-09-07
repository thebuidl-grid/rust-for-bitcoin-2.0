use std::env;
use serde_json::{json, Value};
use anyhow::{bail, Result};
use bdk_wallet::bitcoin::{Block, BlockHash, Transaction, Txid};
use bdk_wallet::bitcoin::consensus::encode::{deserialize_hex, serialize_hex};

pub struct BitRpcClient {
    base_url: String,
    api_key: String,
}

impl BitRpcClient {

    pub fn from_env() -> Result<Self> {
        let base_url = env::var("BITRPC_URL")
            .unwrap_or_else(|_| "https://bitrpc.thebuidl.xyz/bitcoin".to_string());

        let api_key = env::var("BITRPC_API_KEY")
            .map_err(|_| anyhow::anyhow!("BITRPC_API_KEY is not set."))?;

        Ok(Self { base_url, api_key })
    }

    fn call(&self, method: &str, params: Value) -> Result<Value> {
        let body = json!({
            "jsonrpc": "1.0",
            "id": "bitcoin-wallet",
            "method": method,
            "params": params,
        });

        let response = minreq::post(&self.base_url)
            .with_header("Content-Type", "application/json")
            .with_header("X-API-Key", &self.api_key)
            .with_json(&body)?
            .send()?;

         match response.status_code {
            429 => bail!("BitRpc rate limit hit (429) -- slow down and retry shortly"),
            401 => bail!("BitRpc 401 unauthorized -- missing X-API-Key Header"),
            403 =>  bail!("BitRPC 403 -- invalid API key, or `{method}` is not on the allowed list"),
            code if code >= 500 => {
                bail!("BitRPC {code} -- node may be temporarily unavailable, retry shortly")
            }
            _ => {}
        };

        let parsed_result = response.json::<Value>()?;

        if let Some(err) = parsed_result.get("error") {
            if !err.is_null() {
                bail!("RPC error from {method}: {err}");
            }
        }

        parsed_result
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("RPC response missing result"))
    }

    pub fn get_block_count(&self) -> Result<u32> {
        let v = self.call("getblockcount", json!([]))?;
        Ok(v.as_u64().ok_or_else(|| anyhow::anyhow!("get blockcount failed"))? as u32)
    }

    pub fn get_block_hash(&self, height: u32) -> Result<BlockHash> {
        let v = self.call("getblockhash", json!([height]))?;
        Ok(v.as_str().ok_or_else(|| anyhow::anyhow!("get blockhash failed"))?.parse()?)
    }

    pub fn get_block(&self, hash: &BlockHash) -> Result<Block> {
        let v = self.call("getblock", json!([hash.to_string(), 0]))?;
        let hex_str = v.as_str().ok_or_else(|| anyhow::anyhow!("get block failed"))?;
        Ok(deserialize_hex(hex_str)?)
    }

    pub fn send_raw_txn(&self, tx: &Transaction) -> Result<Txid> {
        let hex_str = serialize_hex(tx);
        let v = self.call("sendrawtransaction", json!([hex_str]))?;
        let txid_str = v.as_str().ok_or_else(|| anyhow::anyhow!("bad sendrawtransaction response"))?;
        Ok(txid_str.parse()?)
    }
}