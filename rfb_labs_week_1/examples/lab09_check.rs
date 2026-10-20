use rfb_labs_week_1::labs::lab02_wallets::{create_wallet, get_new_address, list_wallets};
use rfb_labs_week_1::labs::lab03_maturity::mine_blocks;
use rfb_labs_week_1::labs::lab04_utxos::outpoint;
use rfb_labs_week_1::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change,
};
use rfb_labs_week_1::labs::lab09_coin_selection::{
    confirmed_utxos_for_address, create_three_funding_transactions, send_combined_payment,
};
use rfb_labs_week_1::model::MultiUtxoAudit;
use rfb_labs_week_1::rpc::ProcessRpc;

fn main() {
    let rpc = ProcessRpc::new("docker").with_base_args([
        "exec",
        "polar-n3-backend1",
        "bitcoin-cli",
        "-regtest",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    let mining_address = "bcrt1qq6jewkpw6yv97xpxkt8yf2j33p68fhe7kn4sfc";

    if !list_wallets(&rpc).unwrap().contains(&"alice".to_owned()) {
        create_wallet(&rpc, "alice").unwrap();
    }
    let alice_address = get_new_address(&rpc, "alice", "funding").unwrap();
    println!("alice address: {alice_address}");

    let funding_txids = create_three_funding_transactions(&rpc, "miner", &alice_address).unwrap();
    println!("funding txids: {funding_txids:?}");

    mine_blocks(&rpc, mining_address, 1).unwrap();

    let funding_utxos = confirmed_utxos_for_address(&rpc, "alice", &alice_address).unwrap();
    println!("alice's confirmed UTXOs: {funding_utxos:#?}");

    let new_receiver_address = get_new_address(&rpc, "receiver", "lab09-payment").unwrap();
    println!("new receiver address: {new_receiver_address}");

    let spend_txid = send_combined_payment(&rpc, "alice", &new_receiver_address).unwrap();
    mine_blocks(&rpc, mining_address, 1).unwrap();

    let transaction = decode_verbose_transaction(&rpc, &spend_txid).unwrap();
    let funding_outpoints = funding_utxos.iter().map(outpoint).collect();
    let payment_and_change =
        identify_payment_and_change(&transaction, &new_receiver_address).unwrap();
    let fee = calculate_fee(&transaction).unwrap();

    let audit = MultiUtxoAudit {
        funding_outpoints,
        spend_txid,
        spend_input_count: transaction.inputs.len(),
        payment_and_change,
        fee,
    };
    println!("{audit:#?}");
}
