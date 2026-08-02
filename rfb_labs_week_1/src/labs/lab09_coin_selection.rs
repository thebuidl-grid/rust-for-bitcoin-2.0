//! Lab 09 — force and audit multi-UTXO coin selection.

use crate::labs::lab05_mempool::send_btc as send_to_address;
use crate::labs::lab06_decode::{calculate_fee, decode_verbose_transaction, identify_payment_and_change};
use crate::model::{MultiUtxoAudit, OutPoint, Utxo};
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Send three separate 0.4 BTC funding transactions and return their TXIDs.
pub fn create_three_funding_transactions<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    alice_address: &str,
) -> LabResult<Vec<String>> {
    let mut txids = Vec::with_capacity(3);

    // Call send_to_address 3 times for 0.4 BTC each
    for _ in 0..3 {
        let txid = send_to_address(client, miner_wallet, alice_address, 0.4)?;
        txids.push(txid);
    }

    Ok(txids)
}

/// Return confirmed UTXOs belonging to the supplied address.
pub fn confirmed_utxos_for_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<Vec<Utxo>> {
    // 1. Call `listunspent` in the target wallet context: listunspent minconf=1
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let value = parse_cli_value(&raw)?;

    let utxo_array = value
        .as_array()
        .ok_or_else(|| LabError::Parse("listunspent returned non-array JSON".to_owned()))?;

    let mut utxos = Vec::new();

    for item in utxo_array {
        // Filter by matching address if present
        let item_addr = item.get("address").and_then(|a| a.as_str());

        if item_addr == Some(address) {
            let txid = required_string(item, "txid")?;
            let vout = item["vout"]
                .as_u64()
                .ok_or_else(|| LabError::Parse("listunspent item missing 'vout'".to_owned()))?
                as u32;

            let script_pub_key = required_string(item, "scriptPubKey")?;

            let amount = item["amount"]
                .as_f64()
                .ok_or_else(|| LabError::Parse("listunspent item missing 'amount'".to_owned()))?;

            let confirmations = item["confirmations"]
                .as_u64()
                .ok_or_else(|| LabError::Parse("listunspent item missing 'confirmations'".to_owned()))?;

            let spendable = item["spendable"]
                .as_bool()
                .ok_or_else(|| LabError::Parse("listunspent item missing 'spendable'".to_owned()))?;

            utxos.push(Utxo {
                txid,
                vout,
                address: item_addr.map(ToOwned::to_owned),
                script_pub_key,
                amount,
                confirmations,
                spendable,
            });
        }
    }

    Ok(utxos)
}

/// Send 1 BTC from Alice and return the new TXID.
pub fn send_combined_payment<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
) -> LabResult<String> {
    send_to_address(client, alice_wallet, receiver_address, 1.0)
}

/// Decode Alice's spend and prove that multiple funding UTXOs were combined.
pub fn audit_multi_utxo_spend<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
    funding_utxos: &[Utxo],
) -> LabResult<MultiUtxoAudit> {
    // 1. Send the 1 BTC payment from Alice's wallet
    let spend_txid = send_combined_payment(client, alice_wallet, receiver_address)?;

    // 2. Reuse Lab 06 helper to decode the transaction in verbose mode
    let decoded = decode_verbose_transaction(client, &spend_txid)?;

    // 3. Identify payment and change outputs
    let payment_and_change = identify_payment_and_change(&decoded, receiver_address)?;

    // 4. Calculate total miner fee paid
    let fee = calculate_fee(&decoded)?;

    // 5. Collect funding outpoints passed into the function
    let expected_funding_outpoints: Vec<OutPoint> = funding_utxos
        .iter()
        .map(|utxo| OutPoint {
            txid: utxo.txid.clone(),
            vout: utxo.vout,
        })
        .collect();

    let spend_input_count = decoded.inputs.len();

    Ok(MultiUtxoAudit {
        spend_txid,
        spend_input_count,
        payment_and_change,
        fee,
        funding_outpoints: expected_funding_outpoints,
    })
}