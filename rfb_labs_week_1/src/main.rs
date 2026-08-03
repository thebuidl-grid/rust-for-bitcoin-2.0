//! Runner that drives the completed labs against Bitcoin Core nodes in Polar.
//!
//! Every RPC is echoed as the equivalent `bitcoin-cli` command line, so a run of this
//! binary produces the command list and terminal output that `submissions/lab_XX.md`
//! asks for.
//!
//! ```text
//! cargo run -- all      # labs 01-09 against one node, in order, on a fresh network
//! cargo run -- lab05    # one lab (later labs assume earlier ones already ran)
//! cargo run -- lab10    # two-node fork and reorganization
//! ```
//!
//! Configuration comes from the environment:
//!
//! | Variable | Default | Purpose |
//! |---|---|---|
//! | `BITCOIN_CLI` | `bitcoin-cli` | Path to the CLI binary |
//! | `BITCOIN_CLI_ARGS` | `-regtest` | Base arguments for node A |
//! | `NODE_B_CLI_ARGS` | unset | Base arguments for node B (lab 10 only) |
//! | `NODE_B_PEER` | `backend2:18444` | Address node B listens on, for `addnode` |

use rfb_labs_week_1::labs::{
    lab01_network, lab02_wallets, lab03_maturity, lab04_utxos, lab05_mempool, lab06_decode,
    lab07_confirm, lab08_security, lab09_coin_selection, lab10_reorg,
};
use rfb_labs_week_1::model::{ForkSnapshot, Utxo};
use rfb_labs_week_1::rpc::{parse_cli_value, ProcessRpc, RpcClient};
use rfb_labs_week_1::{LabError, LabResult};
use serde::Serialize;
use serde_json::Value;
use std::env;
use std::process::ExitCode;
use std::thread::sleep;
use std::time::Duration;

const MINER_WALLET: &str = "miner";
const RECEIVER_WALLET: &str = "receiver";
const ALICE_WALLET: &str = "alice";

/// Attempts to observe convergence after reconnecting the two nodes in lab 10.
const SYNC_ATTEMPTS: u32 = 30;
const SYNC_INTERVAL: Duration = Duration::from_secs(1);

/// Address node B listens on, as node A resolves it. Polar names the second Bitcoin
/// Core container `backend2` and keeps the regtest P2P port inside the container.
const DEFAULT_NODE_B_PEER: &str = "backend2:18444";

/// [`RpcClient`] that prints the `bitcoin-cli` command line behind every call.
///
/// The lab functions are generic over [`RpcClient`], so wrapping the real transport
/// needs no change to the lab code.
struct LoggingRpc {
    inner: ProcessRpc,
    binary: String,
    base_args: Vec<String>,
}

impl LoggingRpc {
    fn new(binary: String, base_args: Vec<String>) -> Self {
        Self {
            inner: ProcessRpc::new(binary.clone()).with_base_args(base_args.clone()),
            binary,
            base_args,
        }
    }
}

impl RpcClient for LoggingRpc {
    fn call(&self, wallet: Option<&str>, method: &str, params: &[String]) -> LabResult<String> {
        let mut rendered = vec![self.binary.clone()];
        rendered.extend(self.base_args.iter().cloned());
        if let Some(wallet_name) = wallet {
            rendered.push(format!("-rpcwallet={wallet_name}"));
        }
        rendered.push(method.to_owned());
        rendered.extend(params.iter().cloned());
        println!("$ {}", rendered.join(" "));

        let result = self.inner.call(wallet, method, params);
        match &result {
            Ok(output) if output.is_empty() => println!("  (no output)"),
            Ok(output) => {
                for line in output.lines() {
                    println!("  {line}");
                }
            }
            Err(error) => println!("  !! {error}"),
        }
        result
    }
}

fn heading(title: &str) {
    println!("\n{}", "=".repeat(78));
    println!("{title}");
    println!("{}\n", "=".repeat(78));
}

fn report<T: Serialize>(label: &str, value: &T) {
    let rendered = serde_json::to_string_pretty(value)
        .unwrap_or_else(|error| format!("<could not render: {error}>"));
    println!("\n--- {label} ---");
    for line in rendered.lines() {
        println!("{line}");
    }
    println!();
}

/// Load a wallet if it exists, otherwise create it.
fn ensure_wallet<C: RpcClient>(client: &C, name: &str) -> LabResult<()> {
    if lab02_wallets::list_wallets(client)?
        .iter()
        .any(|loaded| loaded == name)
    {
        println!("  wallet `{name}` is already loaded");
        return Ok(());
    }

    match lab02_wallets::create_wallet(client, name) {
        Err(LabError::Rpc(message)) if message.contains("already exists") => {
            client.call(None, "loadwallet", &[name.to_owned()])?;
            Ok(())
        }
        other => other,
    }
}

/// Every connected peer's address, as `disconnectnode` expects it.
///
/// Two nodes that each `addnode` the other hold two connections, one inbound and one
/// outbound. Splitting the network means dropping all of them, not just the first.
fn all_peer_addresses<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "getpeerinfo", &[])?;
    Ok(parse_cli_value(&raw)?
        .as_array()
        .map(|peers| {
            peers
                .iter()
                .filter_map(|peer| peer.get("addr").and_then(Value::as_str))
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default())
}

/// Drop every connection held by one node and report whether it is fully isolated.
fn isolate_node<C: RpcClient>(client: &C, label: &str) -> LabResult<bool> {
    for address in all_peer_addresses(client)? {
        // A peer may already have gone away as the other side disconnects.
        let _ = lab10_reorg::disconnect_peer(client, &address);
    }

    let remaining = all_peer_addresses(client)?;
    if remaining.is_empty() {
        println!("\n{label} is isolated\n");
        Ok(true)
    } else {
        println!("\n!! {label} still has peers: {remaining:?}\n");
        Ok(false)
    }
}

fn env_args(variable: &str, default: &str) -> Vec<String> {
    env::var(variable)
        .unwrap_or_else(|_| default.to_owned())
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect()
}

fn node_a() -> LoggingRpc {
    LoggingRpc::new(
        env::var("BITCOIN_CLI").unwrap_or_else(|_| "bitcoin-cli".to_owned()),
        env_args("BITCOIN_CLI_ARGS", "-regtest"),
    )
}

// ---------------------------------------------------------------------------
// Labs
// ---------------------------------------------------------------------------

fn lab01<C: RpcClient>(client: &C) -> LabResult<()> {
    heading("Lab 01 — build and verify a regtest network");
    report("NetworkSnapshot", &lab01_network::inspect_network(client)?);
    Ok(())
}

/// Returns the mining and classmate addresses used by every later lab.
fn lab02<C: RpcClient>(client: &C) -> LabResult<(String, String)> {
    heading("Lab 02 — create wallets and addresses");

    ensure_wallet(client, MINER_WALLET)?;
    ensure_wallet(client, RECEIVER_WALLET)?;
    println!(
        "\nloaded wallets: {:?}\n",
        lab02_wallets::list_wallets(client)?
    );

    let mining = lab02_wallets::get_new_address(client, MINER_WALLET, "mining")?;
    let classmate = lab02_wallets::get_new_address(client, RECEIVER_WALLET, "classmate")?;

    for (wallet, address) in [(MINER_WALLET, &mining), (RECEIVER_WALLET, &classmate)] {
        let owned = lab02_wallets::address_belongs_to_wallet(client, wallet, address)?;
        println!("\n{wallet} owns {address}: {owned}");
        println!("regtest bcrt1 prefix: {}", address.starts_with("bcrt1"));

        // The negative case is the point of the lab: wallet context decides ownership.
        let other = if wallet == MINER_WALLET {
            RECEIVER_WALLET
        } else {
            MINER_WALLET
        };
        let cross = lab02_wallets::address_belongs_to_wallet(client, other, address)?;
        println!("{other} owns {address}: {cross}\n");
    }

    Ok((mining, classmate))
}

fn lab03<C: RpcClient>(client: &C, mining: &str, classmate: &str) -> LabResult<()> {
    heading("Lab 03 — demonstrate coinbase maturity");
    println!("This lab needs a fresh chain: the premature spend must be rejected.\n");

    let report_value =
        lab03_maturity::demonstrate_coinbase_maturity(client, MINER_WALLET, mining, classmate)?;
    report("CoinbaseMaturityReport", &report_value);
    Ok(())
}

fn lab04<C: RpcClient>(client: &C) -> LabResult<()> {
    heading("Lab 04 — inspect a UTXO and its outpoint");

    let utxos = lab04_utxos::list_unspent(client, MINER_WALLET)?;
    let Some(chosen) = lab04_utxos::select_spendable_utxo(&utxos) else {
        println!("no spendable UTXO yet — run lab03 first");
        return Ok(());
    };

    report("selected Utxo", &chosen);
    report("OutPoint", &lab04_utxos::outpoint(&chosen));

    // Reconcile the independent sum against Bitcoin Core's own balance.
    let summed = lab04_utxos::sum_spendable_utxos(&utxos);
    let trusted = lab03_maturity::get_balances(client, MINER_WALLET)?.trusted;
    println!(
        "spendable UTXO count: {}",
        utxos.iter().filter(|u| u.spendable).count()
    );
    println!("sum(spendable UTXOs) = {summed} BTC");
    println!("getbalances.mine.trusted = {trusted} BTC");
    println!("difference = {} BTC\n", (summed - trusted).abs());
    Ok(())
}

/// Returns the unconfirmed payment TXID for labs 06 and 07.
fn lab05<C: RpcClient>(client: &C, classmate: &str) -> LabResult<String> {
    heading("Lab 05 — broadcast and observe an unconfirmed payment");

    let observation = lab05_mempool::observe_unconfirmed_payment(
        client,
        MINER_WALLET,
        RECEIVER_WALLET,
        classmate,
        1.0,
    )?;
    report("MempoolObservation", &observation);
    println!(
        "broadcast is not confirmation: confirmations = {}\n",
        observation.sender_status.confirmations
    );
    Ok(observation.txid)
}

fn lab06<C: RpcClient>(client: &C, txid: &str, classmate: &str) -> LabResult<()> {
    heading("Lab 06 — decode and audit value conservation");

    let transaction = lab06_decode::decode_verbose_transaction(client, txid)?;
    report("DecodedTransaction", &transaction);
    report(
        "consumed outpoints",
        &lab06_decode::input_outpoints(&transaction),
    );

    let split = lab06_decode::identify_payment_and_change(&transaction, classmate)?;
    report("PaymentAndChange", &split);

    let fee = lab06_decode::calculate_fee(&transaction)?;
    let inputs: f64 = transaction.inputs.iter().map(|i| i.previous_value).sum();
    let change: f64 = split.change.as_ref().map_or(0.0, |output| output.value);

    println!("vsize = {} vB", transaction.vsize);
    println!(
        "fee   = {fee} BTC ({:.2} sat/vB)",
        fee * 100_000_000.0 / transaction.vsize as f64
    );
    println!("\nsum(inputs) = sum(payment) + sum(change) + fee");
    println!("{inputs} = {} + {change} + {fee}\n", split.payment.value);
    Ok(())
}

/// Returns the confirming block hash for lab 08.
fn lab07<C: RpcClient>(client: &C, txid: &str, mining: &str) -> LabResult<String> {
    heading("Lab 07 — confirm and locate the transaction");

    let confirmation =
        lab07_confirm::confirm_and_locate_transaction(client, RECEIVER_WALLET, txid, mining)?;
    report("ConfirmationReport", &confirmation);
    report(
        "receiver balances after confirmation",
        &lab03_maturity::get_balances(client, RECEIVER_WALLET)?,
    );
    Ok(confirmation.block_hash)
}

fn lab08<C: RpcClient>(client: &C, txid: &str, block_hash: &str, mining: &str) -> LabResult<()> {
    heading("Lab 08 — inspect block commitments and confirmation depth");

    let security =
        lab08_security::build_security_report(client, RECEIVER_WALLET, txid, block_hash, mining)?;
    report("SecurityReport", &security);
    println!(
        "confirmations {} -> {} after five blocks\n",
        security.confirmations_before, security.confirmations_after
    );
    Ok(())
}

fn lab09<C: RpcClient>(client: &C, mining: &str) -> LabResult<()> {
    heading("Lab 09 — force multi-UTXO coin selection");

    ensure_wallet(client, ALICE_WALLET)?;
    let alice = lab02_wallets::get_new_address(client, ALICE_WALLET, "funding")?;
    let receiver = lab02_wallets::get_new_address(client, RECEIVER_WALLET, "alice-payment")?;

    let funding =
        lab09_coin_selection::create_three_funding_transactions(client, MINER_WALLET, &alice)?;
    report("funding TXIDs", &funding);

    // Confirm the funding so Alice owns three confirmed, selectable UTXOs.
    lab07_confirm::mine_one_block(client, mining)?;
    let utxos: Vec<Utxo> =
        lab09_coin_selection::confirmed_utxos_for_address(client, ALICE_WALLET, &alice)?;
    report("Alice's confirmed UTXOs", &utxos);
    println!("distinct UTXO count: {}\n", utxos.len());

    let audit =
        lab09_coin_selection::audit_multi_utxo_spend(client, ALICE_WALLET, &receiver, &utxos)?;
    report("MultiUtxoAudit", &audit);
    println!("inputs required: {}", audit.spend_input_count);
    println!("payment: {} BTC", audit.payment_and_change.payment.value);
    if let Some(change) = &audit.payment_and_change.change {
        println!("change:  {} BTC", change.value);
    }
    println!("fee:     {} BTC\n", audit.fee);

    lab07_confirm::mine_one_block(client, mining)?;
    Ok(())
}

fn lab10() -> LabResult<()> {
    heading("Lab 10 — observe competing branches and a reorganization");

    let Ok(node_b_args) = env::var("NODE_B_CLI_ARGS") else {
        println!("Set NODE_B_CLI_ARGS to node B's bitcoin-cli arguments, for example:");
        println!("  NODE_B_CLI_ARGS='-regtest -rpcport=18445' cargo run -- lab10");
        return Ok(());
    };

    let node_a = node_a();
    let node_b = LoggingRpc::new(
        env::var("BITCOIN_CLI").unwrap_or_else(|_| "bitcoin-cli".to_owned()),
        node_b_args
            .split_whitespace()
            .map(ToOwned::to_owned)
            .collect(),
    );

    // Both nodes must agree before the split, or there is no common ancestor to fork.
    let before_a = lab10_reorg::get_chain_tip(&node_a)?;
    let before_b = lab10_reorg::get_chain_tip(&node_b)?;
    report("node A tip before split", &before_a);
    report("node B tip before split", &before_b);
    if before_a.best_block_hash != before_b.best_block_hash {
        println!("!! nodes are not synchronized yet; wait for both tips to match\n");
        return Ok(());
    }
    let common_tip = before_a.best_block_hash.clone();

    // `disconnectnode` needs the live connection address, but `addnode` needs the
    // address node B *listens* on, which is not the same string.
    let node_b_listen = env::var("NODE_B_PEER").unwrap_or_else(|_| DEFAULT_NODE_B_PEER.to_owned());

    // Split the network. Mining into a network that is still connected proves nothing,
    // so stop rather than record a fork that never happened.
    let isolated_a = isolate_node(&node_a, "node A")?;
    let isolated_b = isolate_node(&node_b, "node B")?;
    if !(isolated_a && isolated_b) {
        println!("!! could not fully split the nodes; no fork to observe\n");
        return Ok(());
    }

    ensure_wallet(&node_a, MINER_WALLET)?;
    ensure_wallet(&node_b, MINER_WALLET)?;
    let address_a = lab02_wallets::get_new_address(&node_a, MINER_WALLET, "fork-a")?;
    let address_b = lab02_wallets::get_new_address(&node_b, MINER_WALLET, "fork-b")?;

    // Node B does more work, so its branch must win regardless of arrival order.
    lab03_maturity::mine_blocks(&node_a, &address_a, 2)?;
    lab03_maturity::mine_blocks(&node_b, &address_b, 4)?;

    let competing = ForkSnapshot {
        node_a: lab10_reorg::get_chain_tip(&node_a)?,
        node_b: lab10_reorg::get_chain_tip(&node_b)?,
    };
    report("competing private tips", &competing);
    if competing.node_a.best_block_hash == competing.node_b.best_block_hash {
        println!("!! both nodes share a tip, so the split did not hold\n");
        return Ok(());
    }

    lab10_reorg::reconnect_peer(&node_a, &node_b_listen)?;

    let mut final_tips = ForkSnapshot {
        node_a: lab10_reorg::get_chain_tip(&node_a)?,
        node_b: lab10_reorg::get_chain_tip(&node_b)?,
    };
    for _ in 0..SYNC_ATTEMPTS {
        if final_tips.node_a.best_block_hash == final_tips.node_b.best_block_hash {
            break;
        }
        sleep(SYNC_INTERVAL);
        final_tips = ForkSnapshot {
            node_a: lab10_reorg::get_chain_tip(&node_a)?,
            node_b: lab10_reorg::get_chain_tip(&node_b)?,
        };
    }

    let reorg = lab10_reorg::build_reorg_report(&common_tip, competing, final_tips);
    report("ReorgReport", &reorg);
    println!("converged: {}\n", reorg.converged);
    Ok(())
}

fn run_all() -> LabResult<()> {
    let client = node_a();

    lab01(&client)?;
    let (mining, classmate) = lab02(&client)?;
    lab03(&client, &mining, &classmate)?;
    lab04(&client)?;
    let txid = lab05(&client, &classmate)?;
    lab06(&client, &txid, &classmate)?;
    let block_hash = lab07(&client, &txid, &mining)?;
    lab08(&client, &txid, &block_hash, &mining)?;
    lab09(&client, &mining)?;

    heading("Labs 01-09 complete");
    println!("Run `cargo run -- lab10` with NODE_B_CLI_ARGS set for the reorg lab.\n");
    Ok(())
}

/// Run a single lab, deriving the state it needs from the node.
fn run_one(name: &str) -> LabResult<()> {
    let client = node_a();

    match name {
        "lab01" => lab01(&client),
        "lab02" => lab02(&client).map(|_| ()),
        "lab03" => {
            let (mining, classmate) = lab02(&client)?;
            lab03(&client, &mining, &classmate)
        }
        "lab04" => lab04(&client),
        "lab05" => {
            let classmate = lab02_wallets::get_new_address(&client, RECEIVER_WALLET, "classmate")?;
            lab05(&client, &classmate).map(|_| ())
        }
        "lab06" | "lab07" | "lab08" => {
            let mining = lab02_wallets::get_new_address(&client, MINER_WALLET, "mining")?;
            let classmate = lab02_wallets::get_new_address(&client, RECEIVER_WALLET, "classmate")?;
            let txid = lab05(&client, &classmate)?;
            if name == "lab06" {
                return lab06(&client, &txid, &classmate);
            }
            let block_hash = lab07(&client, &txid, &mining)?;
            if name == "lab07" {
                return Ok(());
            }
            lab08(&client, &txid, &block_hash, &mining)
        }
        "lab09" => {
            let mining = lab02_wallets::get_new_address(&client, MINER_WALLET, "mining")?;
            lab09(&client, &mining)
        }
        "lab10" => lab10(),
        other => {
            println!("unknown lab `{other}`; expected all, or lab01 through lab10");
            Ok(())
        }
    }
}

fn main() -> ExitCode {
    let target = env::args().nth(1).unwrap_or_else(|| "all".to_owned());

    let outcome = match target.as_str() {
        "all" => run_all(),
        name => run_one(name),
    };

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("\n!! {error}");
            eprintln!("Check that bitcoin-cli can reach the intended Polar node, and that the");
            eprintln!("network is fresh if the failure came from lab 03.");
            ExitCode::FAILURE
        }
    }
}
