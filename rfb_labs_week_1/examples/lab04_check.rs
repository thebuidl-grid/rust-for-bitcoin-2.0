use rfb_labs_week_1::labs::lab04_utxos::{list_unspent, outpoint, select_spendable_utxo, sum_spendable_utxos};
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

    let utxos = list_unspent(&rpc, "miner").unwrap();
    println!("miner UTXOs: {utxos:#?}");

    let selected = select_spendable_utxo(&utxos);
    println!("selected spendable UTXO: {selected:#?}");

    if let Some(utxo) = &selected {
        println!("outpoint: {:?}", outpoint(utxo));
    }

    let total = sum_spendable_utxos(&utxos);
    println!("sum of spendable UTXOs: {total}");
}
