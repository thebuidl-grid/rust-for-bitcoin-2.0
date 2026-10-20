//! Lab 09 — force and audit multi-UTXO coin selection.

use crate::model::{MultiUtxoAudit, Utxo};
use crate::rpc::RpcClient;
use crate::LabResult;

use crate::rpc::parse_cli_value;
use crate::LabError;
use crate::labs::lab04_utxos::list_unspent;
use crate::labs::lab06_decode::{
    decode_verbose_transaction, identify_payment_and_change, input_outpoints, calculate_fee,
};

/// Send three separate 0.4 BTC funding transactions and return their TXIDs.
pub fn create_three_funding_transactions<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    alice_address: &str,
) -> LabResult<Vec<String>> {
    let mut txids = Vec::new();
    for _ in 0..3 {
        let raw = client.call(
            Some(miner_wallet),
            "sendtoaddress",
            &[alice_address.to_owned(), "0.4".to_owned()],
        )?;
        let json = parse_cli_value(&raw)?;
        let txid = json.as_str()
            .map(ToOwned::to_owned)
            .ok_or_else(|| LabError::Parse("Expected txid to be a string".to_owned()))?;
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
    let utxos = list_unspent(client, wallet_name)?;
    let filtered = utxos.into_iter()
        .filter(|utxo| utxo.confirmations >= 1 && utxo.address.as_deref() == Some(address))
        .collect();
    Ok(filtered)
}

/// Send 1 BTC from Alice and return the new TXID.
pub fn send_combined_payment<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
) -> LabResult<String> {
    let raw = client.call(
        Some(alice_wallet),
        "sendtoaddress",
        &[receiver_address.to_owned(), "1".to_owned()],
    )?;
    let json = parse_cli_value(&raw)?;
    json.as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse("Expected txid to be a string".to_owned()))
}

/// Decode Alice's spend and prove that multiple funding UTXOs were combined.
pub fn audit_multi_utxo_spend<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
    _funding_utxos: &[Utxo],
) -> LabResult<MultiUtxoAudit> {
    let spend_txid = send_combined_payment(client, alice_wallet, receiver_address)?;
    let decoded = decode_verbose_transaction(client, &spend_txid)?;
    let payment_and_change = identify_payment_and_change(&decoded, receiver_address)?;
    let fee = calculate_fee(&decoded)?;
    let funding_outpoints = input_outpoints(&decoded);
    let spend_input_count = decoded.inputs.len();

    Ok(MultiUtxoAudit {
        funding_outpoints,
        spend_txid,
        spend_input_count,
        payment_and_change,
        fee,
    })
}
