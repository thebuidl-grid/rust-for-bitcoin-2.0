use rfb_labs_week_1::labs::lab01_network::inspect_network;
use rfb_labs_week_1::labs::lab02_wallets::{
    address_belongs_to_wallet, create_wallet, get_new_address, list_wallets,
};
use rfb_labs_week_1::labs::lab03_maturity::{demonstrate_coinbase_maturity, get_balances};
use rfb_labs_week_1::labs::lab04_utxos::{
    list_unspent, outpoint, select_spendable_utxo, sum_spendable_utxos,
};
use rfb_labs_week_1::labs::lab05_mempool::observe_unconfirmed_payment;
use rfb_labs_week_1::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change,
};
use rfb_labs_week_1::labs::lab07_confirm::{confirm_and_locate_transaction, mine_one_block};
use rfb_labs_week_1::labs::lab08_security::build_security_report;
use rfb_labs_week_1::labs::lab09_coin_selection::{
    audit_multi_utxo_spend, confirmed_utxos_for_address, create_three_funding_transactions,
};
use rfb_labs_week_1::rpc::ProcessRpc;
use rfb_labs_week_1::rpc::RpcClient;

fn ensure_wallet<C: RpcClient>(rpc: &C, name: &str) {
    if create_wallet(rpc, name).is_err() {
        // Wallet already exists from a previous run — just load it instead.
        let _ = rpc.call(None, "loadwallet", &[name.to_owned()]);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rpc = ProcessRpc::new("bitcoin-cli").with_base_args([
        "-regtest",
        "-rpcconnect=127.0.0.1",
        "-rpcport=18443",
        "-rpcuser=polaruser",
        "-rpcpassword=polarpass",
    ]);

    println!("=== Lab 01: network ===");
    let snapshot = inspect_network(&rpc)?;
    println!("{snapshot:#?}");

    println!("\n=== Lab 02: wallets ===");
    ensure_wallet(&rpc, "miner");
    ensure_wallet(&rpc, "receiver");
    ensure_wallet(&rpc, "alice");
    println!("wallets: {:?}", list_wallets(&rpc)?);
    let miner_address = get_new_address(&rpc, "miner", "mining")?;
    let receiver_address = get_new_address(&rpc, "receiver", "classmate")?;
    let alice_address = get_new_address(&rpc, "alice", "coinselect")?;
    println!("miner address: {miner_address}");
    println!("receiver address: {receiver_address}");
    println!(
        "miner owns its address: {}",
        address_belongs_to_wallet(&rpc, "miner", &miner_address)?
    );
    println!(
        "receiver owns its address: {}",
        address_belongs_to_wallet(&rpc, "receiver", &receiver_address)?
    );

    println!("\n=== Lab 03: coinbase maturity ===");
    let maturity = demonstrate_coinbase_maturity(&rpc, "miner", &miner_address, &receiver_address)?;
    println!("{maturity:#?}");

    println!("\n=== Lab 04: UTXOs ===");
    let miner_utxos = list_unspent(&rpc, "miner")?;

    if let Some(utxo) = select_spendable_utxo(&miner_utxos) {
        println!("selected UTXO:");
        println!("  txid: {}", utxo.txid);
        println!("  vout: {}", utxo.vout);
        println!("  amount: {}", utxo.amount);
        println!("  confirmations: {}", utxo.confirmations);
        println!("  address: {:?}", utxo.address);
        println!("  script_pub_key: {}", utxo.script_pub_key);
        println!("  spendable: {}", utxo.spendable);
        println!("  outpoint: {:?}", outpoint(&utxo));
    }

    let spendable_total = sum_spendable_utxos(&miner_utxos);
    println!("sum of spendable UTXOs: {spendable_total}");

    let miner_balances = get_balances(&rpc, "miner")?;
    println!("wallet trusted balance: {}", miner_balances.trusted);
    println!(
        "reconciles with wallet balance: {}",
        (spendable_total - miner_balances.trusted).abs() < 0.000_000_01
    );

    println!("\n=== Lab 05: mempool ===");
    let observation =
        observe_unconfirmed_payment(&rpc, "miner", "receiver", &receiver_address, 1.0)?;
    println!("{observation:#?}");

    println!("\n=== Lab 06: decode transaction ===");
    let decoded = decode_verbose_transaction(&rpc, &observation.txid)?;

    println!("vsize: {}", decoded.vsize);

    println!("inputs:");
    for input in &decoded.inputs {
        println!(
            "  {}:{} value: {}",
            input.previous_output.txid, input.previous_output.vout, input.previous_value
        );
    }

    println!("outputs:");
    for output in &decoded.outputs {
        println!(
            "  vout {}: value {} address {:?} script {}",
            output.vout, output.value, output.address, output.script_pub_key_hex
        );
    }

    let payment_and_change = identify_payment_and_change(&decoded, &receiver_address)?;
    println!("{payment_and_change:#?}");

    let fee = calculate_fee(&decoded)?;
    let input_total: f64 = decoded.inputs.iter().map(|i| i.previous_value).sum();
    let output_total: f64 = decoded.outputs.iter().map(|o| o.value).sum();
    println!("fee: {fee}");
    println!("sum(inputs) {input_total} = sum(outputs) {output_total} + fee {fee}");

    println!("\n=== Lab 07: confirm transaction ===");
    let confirmation =
        confirm_and_locate_transaction(&rpc, "receiver", &observation.txid, &miner_address)?;
    println!("{confirmation:#?}");

    println!("\n=== Lab 08: security / block header ===");
    let security = build_security_report(
        &rpc,
        "receiver",
        &observation.txid,
        &confirmation.block_hash,
        &miner_address,
    )?;
    println!("{security:#?}");

    println!("\n=== Lab 09: coin selection ===");
    let funding_txids = create_three_funding_transactions(&rpc, "miner", &alice_address)?;
    println!("funding txids: {funding_txids:?}");
    mine_one_block(&rpc, &miner_address)?; // confirm the 3 funding txs
    let funding_utxos = confirmed_utxos_for_address(&rpc, "alice", &alice_address)?;
    println!("alice's confirmed UTXOs:");
    for utxo in &funding_utxos {
        println!(
            "  {}:{} amount {} confirmations {}",
            utxo.txid, utxo.vout, utxo.amount, utxo.confirmations
        );
    }
    let audit = audit_multi_utxo_spend(&rpc, "alice", &receiver_address, &funding_utxos)?;
    println!("{audit:#?}");

    println!("\nAll labs 1-9 ran against the live node.");
    Ok(())
}
