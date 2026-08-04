//! Run every Week 1 lab end to end against live Bitcoin Core regtest nodes.
//!
//! Node A carries labs 01–09. Lab 10 additionally needs node B so the two can mine
//! competing private branches.
//!
//! ```bash
//! RFB_NODE_A="-regtest -datadir=/path/node-a" \
//! RFB_NODE_B="-regtest -datadir=/path/node-b" \
//! RFB_PEER_B="127.0.0.1:18455" \
//! cargo run --example week1_walkthrough
//! ```

use rfb_labs_week_1::labs::{
    lab01_network, lab02_wallets, lab03_maturity, lab04_utxos, lab05_mempool, lab06_decode,
    lab07_confirm, lab08_security, lab09_coin_selection, lab10_reorg,
};
use rfb_labs_week_1::model::ForkSnapshot;
use rfb_labs_week_1::rpc::ProcessRpc;
use rfb_labs_week_1::{LabError, LabResult};
use std::env;
use std::thread::sleep;
use std::time::Duration;

fn main() -> LabResult<()> {
    let node_a = ProcessRpc::new("bitcoin-cli").with_base_args(cli_args("RFB_NODE_A"));
    let node_b = ProcessRpc::new("bitcoin-cli").with_base_args(cli_args("RFB_NODE_B"));
    // Node A always dials node B. Only the node that opened the connection knows the
    // peer by its listening address, and `disconnectnode` matches on that address.
    let peer_b = peer("RFB_PEER_B", "127.0.0.1:18455");

    // ---------------------------------------------------------------- Lab 01
    section("Lab 01 — regtest network inspection");
    let snapshot = lab01_network::inspect_network(&node_a)?;
    println!("chain            = {}", snapshot.chain);
    println!("block height     = {}", snapshot.block_height);
    println!("best block hash  = {}", snapshot.best_block_hash);

    // ---------------------------------------------------------------- Lab 02
    section("Lab 02 — wallets and addresses");
    lab02_wallets::create_wallet(&node_a, "miner")?;
    lab02_wallets::create_wallet(&node_a, "receiver")?;
    println!(
        "loaded wallets   = {:?}",
        lab02_wallets::list_wallets(&node_a)?
    );

    let mining_address = lab02_wallets::get_new_address(&node_a, "miner", "mining")?;
    let classmate_address = lab02_wallets::get_new_address(&node_a, "receiver", "classmate")?;
    println!("mining address   = {mining_address}");
    println!("classmate addr   = {classmate_address}");
    println!(
        "mining addr is miner's    = {}",
        lab02_wallets::address_belongs_to_wallet(&node_a, "miner", &mining_address)?
    );
    println!(
        "mining addr is receiver's = {}",
        lab02_wallets::address_belongs_to_wallet(&node_a, "receiver", &mining_address)?
    );
    println!(
        "classmate addr is receiver's = {}",
        lab02_wallets::address_belongs_to_wallet(&node_a, "receiver", &classmate_address)?
    );

    // ---------------------------------------------------------------- Lab 03
    section("Lab 03 — coinbase maturity");
    let maturity = lab03_maturity::demonstrate_coinbase_maturity(
        &node_a,
        "miner",
        &mining_address,
        &classmate_address,
    )?;
    println!(
        "height after 1 block = {}",
        maturity.height_after_first_block
    );
    println!(
        "balances after 1     = {:?}",
        maturity.balance_after_first_block
    );
    println!("premature spend error= {}", maturity.premature_spend_error);
    println!("final height         = {}", maturity.final_height);
    println!("final balances       = {:?}", maturity.final_balance);

    // ---------------------------------------------------------------- Lab 04
    section("Lab 04 — UTXOs and outpoints");
    let utxos = lab04_utxos::list_unspent(&node_a, "miner")?;
    println!("wallet UTXO count = {}", utxos.len());

    let chosen = lab04_utxos::select_spendable_utxo(&utxos)
        .ok_or_else(|| LabError::Parse("the miner wallet has no spendable UTXO".to_owned()))?;
    println!("txid          = {}", chosen.txid);
    println!("vout          = {}", chosen.vout);
    println!("amount        = {} BTC", chosen.amount);
    println!("confirmations = {}", chosen.confirmations);
    println!("address       = {:?}", chosen.address);
    println!("scriptPubKey  = {}", chosen.script_pub_key);
    println!("spendable     = {}", chosen.spendable);

    let point = lab04_utxos::outpoint(&chosen);
    println!("outpoint      = {}:{}", point.txid, point.vout);
    println!(
        "sum(spendable UTXOs) = {} BTC",
        lab04_utxos::sum_spendable_utxos(&utxos)
    );
    println!(
        "wallet trusted balance = {} BTC",
        lab03_maturity::get_balances(&node_a, "miner")?.trusted
    );

    // ---------------------------------------------------------------- Lab 05
    section("Lab 05 — broadcast and mempool state");
    let observation = lab05_mempool::observe_unconfirmed_payment(
        &node_a,
        "miner",
        "receiver",
        &classmate_address,
        1.0,
    )?;
    println!("txid               = {}", observation.txid);
    println!("in local mempool   = {}", observation.mempool_contains_tx);
    println!("sender status      = {:?}", observation.sender_status);
    println!("receiver balances  = {:?}", observation.receiver_balance);

    // ---------------------------------------------------------------- Lab 06
    section("Lab 06 — decoding and value conservation");
    let decoded = lab06_decode::decode_verbose_transaction(&node_a, &observation.txid)?;
    println!("txid  = {}", decoded.txid);
    println!("vsize = {} vB", decoded.vsize);
    for point in lab06_decode::input_outpoints(&decoded) {
        println!("consumes {}:{}", point.txid, point.vout);
    }
    for input in &decoded.inputs {
        println!(
            "input  {}:{} = {} BTC",
            input.previous_output.txid, input.previous_output.vout, input.previous_value
        );
    }
    for output in &decoded.outputs {
        println!(
            "output vout {} = {} BTC to {:?}",
            output.vout, output.value, output.address
        );
    }

    let split = lab06_decode::identify_payment_and_change(&decoded, &classmate_address)?;
    let fee = lab06_decode::calculate_fee(&decoded)?;
    let input_total: f64 = decoded
        .inputs
        .iter()
        .map(|input| input.previous_value)
        .sum();
    let change_total = split.change.as_ref().map_or(0.0, |output| output.value);
    println!(
        "payment = {} BTC (vout {})",
        split.payment.value, split.payment.vout
    );
    println!("change  = {change_total} BTC");
    println!("fee     = {fee} BTC");
    println!(
        "conservation: {input_total} = {} + {change_total} + {fee}",
        split.payment.value
    );

    // ---------------------------------------------------------------- Lab 07
    section("Lab 07 — confirmation and block membership");
    let confirmation = lab07_confirm::confirm_and_locate_transaction(
        &node_a,
        "receiver",
        &observation.txid,
        &mining_address,
    )?;
    println!("txid                 = {}", confirmation.txid);
    println!("mempool is empty     = {}", confirmation.mempool_is_empty);
    println!("confirmations        = {}", confirmation.confirmations);
    println!("containing block     = {}", confirmation.block_hash);
    println!(
        "block contains txid  = {}",
        confirmation.transaction_is_in_block
    );
    println!(
        "receiver balances    = {:?}",
        lab03_maturity::get_balances(&node_a, "receiver")?
    );

    // ---------------------------------------------------------------- Lab 08
    section("Lab 08 — headers, proof of work, and depth");
    let security = lab08_security::build_security_report(
        &node_a,
        "receiver",
        &observation.txid,
        &confirmation.block_hash,
        &mining_address,
    )?;
    let header = &security.header;
    println!("block hash      = {}", header.hash);
    println!("height          = {}", header.height);
    println!("previous hash   = {:?}", header.previous_block_hash);
    println!("merkle root     = {}", header.merkle_root);
    println!("nonce           = {}", header.nonce);
    println!("bits            = {}", header.bits);
    println!("difficulty      = {}", header.difficulty);
    println!("confirmations   = {}", header.confirmations);
    println!("chainwork       = {}", header.chainwork);
    println!("depth before    = {}", security.confirmations_before);
    println!("depth after +5  = {}", security.confirmations_after);

    // ---------------------------------------------------------------- Lab 09
    section("Lab 09 — multi-UTXO coin selection");
    lab02_wallets::create_wallet(&node_a, "alice")?;
    let alice_address = lab02_wallets::get_new_address(&node_a, "alice", "alice-funding")?;
    let alice_payee = lab02_wallets::get_new_address(&node_a, "receiver", "alice-payment")?;
    println!("alice address   = {alice_address}");

    let funding_txids =
        lab09_coin_selection::create_three_funding_transactions(&node_a, "miner", &alice_address)?;
    println!("funding txids   = {funding_txids:?}");
    lab03_maturity::mine_blocks(&node_a, &mining_address, 1)?;

    let funding_utxos =
        lab09_coin_selection::confirmed_utxos_for_address(&node_a, "alice", &alice_address)?;
    println!("alice UTXO count = {}", funding_utxos.len());
    for utxo in &funding_utxos {
        println!("  {}:{} = {} BTC", utxo.txid, utxo.vout, utxo.amount);
    }

    let audit = lab09_coin_selection::audit_multi_utxo_spend(
        &node_a,
        "alice",
        &alice_payee,
        &funding_utxos,
    )?;
    println!("spend txid      = {}", audit.spend_txid);
    println!("inputs consumed = {}", audit.spend_input_count);
    println!(
        "payment         = {} BTC to {:?}",
        audit.payment_and_change.payment.value, audit.payment_and_change.payment.address
    );
    println!(
        "change          = {:?}",
        audit
            .payment_and_change
            .change
            .as_ref()
            .map(|out| out.value)
    );
    println!("fee             = {} BTC", audit.fee);
    lab03_maturity::mine_blocks(&node_a, &mining_address, 1)?;

    // ---------------------------------------------------------------- Lab 10
    section("Lab 10 — competing branches and the most-work rule");
    lab10_reorg::reconnect_peer(&node_a, &peer_b)?;
    let common = wait_for_convergence(&node_a, &node_b)?;
    println!("common height   = {}", common.node_a.height);
    println!("common tip      = {}", common.node_a.best_block_hash);
    println!("common chainwork= {}", common.node_a.chainwork);

    // Dropping the link from one side is enough: the peers share a single TCP
    // connection, so node B loses node A at the same moment. Asking node B to
    // disconnect afterwards would fail with "Node not found in connected nodes".
    lab10_reorg::disconnect_peer(&node_a, &peer_b)?;
    sleep(Duration::from_secs(1));
    println!("nodes disconnected");

    lab02_wallets::create_wallet(&node_b, "node-b-miner")?;
    let node_b_address = lab02_wallets::get_new_address(&node_b, "node-b-miner", "node-b-mining")?;
    lab03_maturity::mine_blocks(&node_a, &mining_address, 2)?;
    lab03_maturity::mine_blocks(&node_b, &node_b_address, 4)?;

    let competing = ForkSnapshot {
        node_a: lab10_reorg::get_chain_tip(&node_a)?,
        node_b: lab10_reorg::get_chain_tip(&node_b)?,
    };
    println!(
        "node A private tip = {} at height {} (chainwork {})",
        competing.node_a.best_block_hash, competing.node_a.height, competing.node_a.chainwork
    );
    println!(
        "node B private tip = {} at height {} (chainwork {})",
        competing.node_b.best_block_hash, competing.node_b.height, competing.node_b.chainwork
    );

    lab10_reorg::reconnect_peer(&node_a, &peer_b)?;
    let final_tips = wait_for_convergence(&node_a, &node_b)?;
    println!(
        "node A final tip   = {} at height {}",
        final_tips.node_a.best_block_hash, final_tips.node_a.height
    );
    println!(
        "node B final tip   = {} at height {}",
        final_tips.node_b.best_block_hash, final_tips.node_b.height
    );

    let report = lab10_reorg::build_reorg_report(
        &common.node_a.best_block_hash,
        competing.clone(),
        final_tips,
    );
    println!("converged          = {}", report.converged);
    println!(
        "stale branch       = {} (node A's branch, {} blocks above the common tip, \
         discarded for node B's greater accumulated work)",
        competing.node_a.best_block_hash,
        competing.node_a.height - common.node_a.height
    );

    Ok(())
}

/// Poll both nodes until they agree on a tip, so mining and relay have time to settle.
fn wait_for_convergence(node_a: &ProcessRpc, node_b: &ProcessRpc) -> LabResult<ForkSnapshot> {
    for _ in 0..60 {
        let tips = ForkSnapshot {
            node_a: lab10_reorg::get_chain_tip(node_a)?,
            node_b: lab10_reorg::get_chain_tip(node_b)?,
        };

        if tips.node_a.best_block_hash == tips.node_b.best_block_hash {
            return Ok(tips);
        }

        sleep(Duration::from_millis(500));
    }

    Err(LabError::Rpc(
        "the two nodes did not converge on a common tip".to_owned(),
    ))
}

/// Read whitespace-separated `bitcoin-cli` arguments from the environment.
fn cli_args(key: &str) -> Vec<String> {
    env::var(key)
        .unwrap_or_else(|_| "-regtest".to_owned())
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect()
}

/// Read a peer `host:port` from the environment.
fn peer(key: &str, fallback: &str) -> String {
    env::var(key).unwrap_or_else(|_| fallback.to_owned())
}

fn section(title: &str) {
    println!("\n========== {title} ==========");
}
