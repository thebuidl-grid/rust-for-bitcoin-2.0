//! Lab 03 — demonstrate coinbase maturity.

use crate::labs::lab01_network::get_block_height;
use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    // TODO: call generatetoaddress with count and address.
    // todo!("Lab 03: mine blocks")
    let result = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_string()],
    );

    match result {
        Ok(res) => {
            let parsed = parse_cli_value(&res)?;
            let vector_of_result = parsed
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect::<Vec<String>>();

            Ok(vector_of_result)
        }
        Err(err) => Err(err),
    }
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    // TODO: call getbalances in wallet context and decode the nested `mine` object.
    // todo!("Lab 03: inspect wallet balances")

    let result = if wallet_name.is_empty() {
        client.call(None, "getbalances", &[])
    } else {
        client.call(Some(wallet_name), "getbalances", &[])
    };

    match result {
        Ok(res) => {
            let parsed = parse_cli_value(&res)?;
            let mine = &parsed["mine"];

            match serde_json::from_value::<WalletBalances>(mine.clone()) {
                Ok(result) => Ok(result),
                Err(err) => Err(err.into()),
            }
        }
        Err(err) => Err(err),
    }
}

/// Attempt a wallet payment and return either its TXID or the Bitcoin Core error.
pub fn attempt_payment<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
    amount_btc: f64,
) -> LabResult<String> {
    // TODO: call sendtoaddress. Do not hide an insufficient-funds RPC error.
    // todo!("Lab 03: attempt a payment")

    let result = client.call(
        Some(wallet_name),
        "sendtoaddress",
        &[address.to_string(), amount_btc.to_string()],
    );

    match result {
        Ok(res) => Ok(res),
        Err(err) => Err(err.into()),
    }
}

/// Mine one block, prove the reward is immature, then mine 100 more blocks.
pub fn demonstrate_coinbase_maturity<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    miner_address: &str,
    receiver_address: &str,
) -> LabResult<CoinbaseMaturityReport> {
    // TODO:
    // 1. Mine one block.
    // 2. Record height and balances.
    // 3. Attempt a 1 BTC payment and capture its error text.
    // 4. Mine 100 more blocks.
    // 5. Record final height and balances.
    // todo!("Lab 03: produce coinbase-maturity evidence")
    mine_blocks(client, miner_address, 1)?;
    let height_after_one = get_block_height(client)?;
    let balance_after_one_mined_block = get_balances(client, miner_wallet)?;

    let payment_attempted = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
        Ok(_) => "".to_string(),
        Err(LabError::Rpc(msg)) => msg,
        Err(err) => err.to_string(),
    };
    mine_blocks(client, miner_address, 100)?;
    let height_after_hundred = get_block_height(client)?;
    let balance_after_hundred_mined_block = get_balances(client, miner_wallet)?;

    let result = CoinbaseMaturityReport {
        height_after_first_block: height_after_one,
        balance_after_first_block: balance_after_one_mined_block,
        premature_spend_error: payment_attempted.to_string(),
        final_height: height_after_hundred,
        final_balance: balance_after_hundred_mined_block,
    };

    Ok(result)
}
