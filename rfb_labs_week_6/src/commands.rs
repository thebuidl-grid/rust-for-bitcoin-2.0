//! One function per subcommand.

use bdk_wallet::chain::ChainPosition;
use bdk_wallet::{KeychainKind, Wallet};
use bitcoin::{Address, Amount, FeeRate, Txid};

use crate::cli::{AddressArgs, InitArgs, MineArgs, SendArgs, SyncArgs, UtxosArgs, VerifyArgs};
use crate::config::Config;
use crate::error::{Error, Result};
use crate::keys::{self, DescriptorKind};
use crate::tx::{self, SpendRequest};
use crate::wallet::{self, WalletHandle};
use crate::{node, raw, store, sync, ui};

pub fn init(cfg: &Config, args: &InitArgs) -> Result<()> {
    // Ask the node where the chain is now so the first sync does not have to
    // walk history the wallet cannot possibly own.
    let (birthday, node_note) = match args.birthday {
        Some(height) => (height, format!("supplied on the command line ({height})")),
        None => match node::connect(cfg).and_then(|c| node::block_count(&c)) {
            Ok(height) => (height, format!("current node height ({height})")),
            Err(e) => (0, format!("node unreachable ({e}); defaulting to 0")),
        },
    };

    let mut handle = wallet::create(cfg, args.descriptor, birthday)?;
    let external = handle.wallet.reveal_next_address(KeychainKind::External);
    handle.persist()?;

    ui::heading("wallet created");
    ui::row("database", cfg.db_path.display());
    ui::row("network", cfg.network);
    ui::row(
        "descriptor",
        format!("{} ({})", args.descriptor.as_str(), args.descriptor.bip()),
    );
    ui::row("birthday", node_note);

    print_descriptors(&handle);

    ui::heading("first receive address");
    ui::row("index", external.index);
    ui::row("address", external.address);

    ui::note(
        "The database holds public descriptors only. Keep RFB_MNEMONIC in your .env:\n\
         it is the only copy of the private keys, and every command that signs rebuilds\n\
         them from it at runtime.",
    );
    Ok(())
}

pub fn info(cfg: &Config) -> Result<()> {
    let handle = wallet::load(cfg)?;

    ui::heading("wallet");
    ui::row("database", cfg.db_path.display());
    ui::row(
        "size on disk",
        wallet::db_size(&cfg.db_path)
            .map(ui::bytes)
            .unwrap_or_else(|| "unknown".into()),
    );
    ui::row("network", handle.wallet.network());
    ui::row(
        "descriptor",
        format!("{} ({})", handle.kind.as_str(), handle.kind.bip()),
    );
    ui::row(
        "master fingerprint",
        wallet::recorded_fingerprint(&handle.conn)?
            .map(|f| f.to_string())
            .unwrap_or_else(|| "not recorded".into()),
    );
    ui::row("can sign", ui::yes_no(handle.signing));
    ui::row("birthday height", store::birthday(&handle.conn)?);

    ui::heading("keychains");
    let mut rows = Vec::new();
    for keychain in [KeychainKind::External, KeychainKind::Internal] {
        rows.push(vec![
            keychain_name(keychain).to_string(),
            handle
                .wallet
                .derivation_index(keychain)
                .map(|i| i.to_string())
                .unwrap_or_else(|| "none revealed".into()),
            handle.wallet.next_derivation_index(keychain).to_string(),
            handle.wallet.descriptor_checksum(keychain),
        ]);
    }
    ui::table(
        &["keychain", "last revealed", "next index", "checksum"],
        &rows,
    );

    print_descriptors(&handle);

    ui::heading("chain state");
    let cp = handle.wallet.latest_checkpoint();
    ui::row("checkpoint height", cp.height());
    ui::row("checkpoint hash", cp.hash());
    ui::row("transactions", handle.wallet.transactions().count());
    ui::row("unspent outputs", handle.wallet.list_unspent().count());
    Ok(())
}

pub fn address(cfg: &Config, args: &AddressArgs) -> Result<()> {
    let mut handle = wallet::load(cfg)?;
    let keychain = if args.change {
        KeychainKind::Internal
    } else {
        KeychainKind::External
    };

    let info = match (args.peek, args.unused) {
        (Some(index), _) => handle.wallet.peek_address(keychain, index),
        (None, true) => handle.wallet.next_unused_address(keychain),
        (None, false) => handle.wallet.reveal_next_address(keychain),
    };

    // Peeking does not change wallet state; revealing does and must be stored.
    let persisted = handle.persist()?;

    ui::heading(match (args.peek, args.unused) {
        (Some(_), _) => "peeked address",
        (None, true) => "next unused address",
        (None, false) => "new address",
    });
    ui::row("keychain", keychain_name(keychain));
    ui::row("index", info.index);
    ui::row("address", &info.address);
    ui::row(
        "script pubkey",
        info.address.script_pubkey().to_hex_string(),
    );
    ui::row("persisted", ui::yes_no(persisted));
    Ok(())
}

pub fn balance(cfg: &Config) -> Result<()> {
    let handle = wallet::load(cfg)?;
    let balance = handle.wallet.balance();

    ui::heading("balance");
    ui::row("confirmed", ui::sats(balance.confirmed));
    ui::row("trusted pending", ui::sats(balance.trusted_pending));
    ui::row("untrusted pending", ui::sats(balance.untrusted_pending));
    ui::row("immature", ui::sats(balance.immature));
    ui::row("spendable now", ui::sats(balance.trusted_spendable()));
    ui::row("total", ui::sats(balance.total()));

    if balance.immature > Amount::ZERO {
        ui::note(
            "Immature funds are coinbase outputs. They need 100 confirmations before\n\
             they can be spent; on regtest, mine 100 more blocks.",
        );
    }
    Ok(())
}

pub fn utxos(cfg: &Config, args: &UtxosArgs) -> Result<()> {
    let handle = wallet::load(cfg)?;
    let tip = handle.wallet.latest_checkpoint().height();

    let mut utxos: Vec<_> = handle.wallet.list_unspent().collect();
    // Biggest first, then oldest first, so the outputs that are actually
    // spendable rise to the top of a wallet full of fresh coinbases.
    utxos.sort_by_key(|u| {
        let height = match &u.chain_position {
            ChainPosition::Confirmed { anchor, .. } => anchor.block_id.height,
            ChainPosition::Unconfirmed { .. } => u32::MAX,
        };
        (std::cmp::Reverse(u.txout.value), height)
    });

    ui::heading("unspent outputs");
    if utxos.is_empty() {
        ui::row(
            "",
            "none — sync the wallet, or mine to one of its addresses",
        );
        return Ok(());
    }

    // A regtest wallet that mined its own coins has hundreds of near-identical
    // coinbase outputs. Show the largest few unless asked for everything.
    let shown: Vec<_> = if args.all {
        utxos.iter().collect()
    } else {
        utxos.iter().take(DEFAULT_UTXO_ROWS).collect()
    };

    let rows: Vec<Vec<String>> = shown
        .iter()
        .map(|u| {
            vec![
                format!("{}:{}", u.outpoint.txid, u.outpoint.vout),
                u.txout.value.to_sat().to_string(),
                keychain_name(u.keychain).to_string(),
                u.derivation_index.to_string(),
                utxo_status(&handle, u, tip),
            ]
        })
        .collect();

    ui::table(&["outpoint", "sat", "keychain", "index", "status"], &rows);

    let total: Amount = utxos.iter().map(|u| u.txout.value).sum();
    ui::blank();
    if shown.len() < utxos.len() {
        ui::row(
            "showing",
            format!(
                "{} of {} (pass --all for the rest)",
                shown.len(),
                utxos.len()
            ),
        );
    }
    ui::row("count", utxos.len());
    ui::row("total", ui::sats(total));
    ui::row(
        "spendable",
        ui::sats(handle.wallet.balance().trusted_spendable()),
    );
    Ok(())
}

pub fn txs(cfg: &Config) -> Result<()> {
    let handle = wallet::load(cfg)?;
    let mut txs: Vec<_> = handle.wallet.transactions().collect();
    txs.sort_by_key(|tx| match &tx.chain_position {
        ChainPosition::Confirmed { anchor, .. } => anchor.block_id.height,
        ChainPosition::Unconfirmed { .. } => u32::MAX,
    });

    ui::heading("transactions");
    if txs.is_empty() {
        ui::row("", "none yet");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = txs
        .iter()
        .map(|wtx| {
            let (sent, received) = handle.wallet.sent_and_received(&wtx.tx_node.tx);
            let net = received.to_sat() as i64 - sent.to_sat() as i64;
            vec![
                wtx.tx_node.txid.to_string(),
                position(&wtx.chain_position),
                format!("{net:+}"),
                if wtx.tx_node.tx.is_coinbase() {
                    // A coinbase has no inputs to pay a fee from; reporting 0
                    // would read as "this transaction paid nothing".
                    "n/a".to_string()
                } else {
                    handle
                        .wallet
                        .calculate_fee(&wtx.tx_node.tx)
                        .map(|f| f.to_sat().to_string())
                        .unwrap_or_else(|_| "-".into())
                },
            ]
        })
        .collect();

    ui::table(&["txid", "status", "net sat", "fee sat"], &rows);
    Ok(())
}

pub fn sync_cmd(cfg: &Config, args: &SyncArgs) -> Result<()> {
    let client = node::connect(cfg)?;
    let mut handle = wallet::load(cfg)?;

    let start = args.from_height.unwrap_or(store::birthday(&handle.conn)?);

    let before = handle.wallet.balance();
    let summary = sync::sync(&mut handle, &client, start)?;
    let after = handle.wallet.balance();

    ui::heading("sync");
    ui::row("start height", summary.start_height);
    ui::row("blocks applied", summary.blocks_applied);
    ui::row("tip height", summary.tip_height);
    ui::row("mempool txs seen", summary.mempool_txs);
    ui::row("mempool evictions", summary.evicted_txs);

    ui::heading("balance");
    ui::row("before", ui::sats(before.total()));
    ui::row("after", ui::sats(after.total()));
    ui::row("spendable now", ui::sats(after.trusted_spendable()));
    Ok(())
}

pub fn send(cfg: &Config, args: &SendArgs) -> Result<()> {
    let client = node::connect(cfg)?;
    let mut handle = wallet::load(cfg)?;
    handle.require_signing()?;

    let recipient = args
        .to
        .parse::<Address<_>>()?
        .require_network(cfg.network)
        .map_err(|_| Error::AddressNetworkMismatch {
            address: args.to.clone(),
            network: cfg.network,
        })?;

    if !args.drain && args.amount == 0 {
        return Err(Error::Config(
            "--amount must be greater than zero, or pass --drain to sweep the wallet".into(),
        ));
    }

    let fee_rate = FeeRate::from_sat_per_vb(args.fee_rate).ok_or_else(|| {
        Error::Config(format!("fee rate {} sat/vB is out of range", args.fee_rate))
    })?;

    let request = SpendRequest {
        recipient: recipient.clone(),
        amount: Amount::from_sat(args.amount),
        fee_rate,
        selection: args.selection,
        drain: args.drain,
    };

    let draft = tx::build_and_sign(&mut handle, &request)?;
    // Building revealed a change address; store it before anything can fail.
    handle.persist()?;

    ui::heading("transaction");
    ui::row("txid", draft.tx.compute_txid());
    ui::row("recipient", &recipient);
    ui::row(
        "amount",
        if args.drain {
            "whole spendable balance".to_string()
        } else {
            ui::sats(request.amount)
        },
    );
    ui::row("coin selection", args.selection.as_str());
    ui::row("inputs", draft.inputs);
    ui::row("outputs", draft.outputs);
    ui::row(
        "change",
        draft
            .change
            .map(ui::sats)
            .unwrap_or_else(|| "none (changeless solution)".into()),
    );
    ui::row("fee", ui::sats(draft.fee));
    ui::row("effective fee rate", ui::fee_rate(draft.fee_rate));
    ui::row("weight", format!("{} wu", draft.tx.weight().to_wu()));
    ui::row("virtual size", format!("{} vB", draft.tx.vsize()));

    if args.dry_run {
        ui::heading("signed psbt (base64)");
        ui::note(draft.psbt.to_string());
        ui::note("--dry-run: signed but not broadcast.");
        return Ok(());
    }

    let txid = node::broadcast(&client, &draft.tx)?;

    // Record the broadcast transaction locally so `balance` reflects it before
    // the next sync, then let sync reconcile it against the node's mempool.
    handle
        .wallet
        .apply_unconfirmed_txs([(draft.tx.clone(), wallet::unix_now())]);
    handle.persist()?;

    ui::heading("broadcast");
    ui::row("txid", txid);
    ui::row("relayed to", &cfg.rpc_url);
    ui::note(format!(
        "Verify it independently with:\n  rfbwallet verify {txid}"
    ));
    Ok(())
}

pub fn node_info(cfg: &Config) -> Result<()> {
    let client = node::connect(cfg)?;
    let info = node::info(&client)?;

    ui::heading("node");
    ui::row("rpc url", &cfg.rpc_url);
    ui::row("authentication", format!("{:?}", cfg.rpc_auth));
    ui::row("version", &info.subversion);
    ui::row("chain", info.chain);
    ui::row("blocks", info.blocks);
    ui::row("headers", info.headers);
    ui::row("best block", info.best_block_hash);
    ui::row("in ibd", ui::yes_no(info.initial_block_download));
    ui::row("peers", info.connections);
    ui::row(
        "fee estimate 6 blk",
        node::fee_estimate(&client, 6)
            .map(|a| format!("{} sat/kvB", a.to_sat()))
            .unwrap_or_else(|| "unavailable (normal on regtest)".into()),
    );
    Ok(())
}

pub fn mine(cfg: &Config, args: &MineArgs) -> Result<()> {
    let client = node::connect(cfg)?;
    let mut handle = wallet::load(cfg)?;

    let address = match &args.to {
        Some(raw) => raw
            .parse::<Address<_>>()?
            .require_network(cfg.network)
            .map_err(|_| Error::AddressNetworkMismatch {
                address: raw.clone(),
                network: cfg.network,
            })?,
        None => {
            let info = handle.wallet.reveal_next_address(KeychainKind::External);
            handle.persist()?;
            info.address
        }
    };

    let hashes = node::mine_to(&client, cfg.network, &address, args.blocks)?;

    ui::heading("mined");
    ui::row("blocks", hashes.len());
    ui::row("to address", &address);
    if let Some(first) = hashes.first() {
        ui::row("first block", first);
    }
    if let Some(last) = hashes.last() {
        ui::row("tip block", last);
    }
    ui::row("node height", node::block_count(&client)?);
    ui::note("Run `rfbwallet sync` to pull these blocks into the wallet.");
    Ok(())
}

pub fn verify(cfg: &Config, args: &VerifyArgs) -> Result<()> {
    let handle = wallet::load(cfg)?;
    let txid: Txid = args
        .txid
        .parse()
        .map_err(|e| Error::wallet("parsing the transaction id", e))?;

    // The node is preferred but not required: a wallet transaction can be
    // verified from the local graph alone.
    let client = node::connect(cfg).ok();
    let report = raw::verify(&handle, client.as_ref(), txid)?;

    ui::heading("decoded transaction");
    ui::row("source", report.source.as_str());
    ui::row("raw size", format!("{} B", report.raw_len));
    ui::row("version", report.version);
    ui::row("lock time", report.lock_time);
    ui::row("weight", format!("{} wu", report.weight.to_wu()));
    ui::row("virtual size", format!("{} vB", report.vsize));
    ui::row("wtxid", report.wtxid);

    ui::heading("txid check");
    ui::row("requested", report.declared_txid);
    ui::row("recomputed", report.computed_txid);
    ui::row(
        "match",
        ui::yes_no(report.declared_txid == report.computed_txid),
    );

    ui::heading("inputs");
    let rows: Vec<Vec<String>> = report
        .inputs
        .iter()
        .map(|i| {
            vec![
                i.index.to_string(),
                format!("{}:{}", i.outpoint.txid, i.outpoint.vout),
                i.value.to_sat().to_string(),
                i.spend_type.to_string(),
                match i.signature_valid {
                    Some(true) => "valid".to_string(),
                    Some(false) => "INVALID".to_string(),
                    None => "not checked".to_string(),
                },
            ]
        })
        .collect();
    ui::table(&["#", "spends", "sat", "type", "signature"], &rows);
    for input in &report.inputs {
        if let Some(note) = &input.note {
            ui::row(&format!("input {}", input.index), note);
        }
    }

    ui::heading("outputs");
    let rows: Vec<Vec<String>> = report
        .outputs
        .iter()
        .map(|(i, value, kind)| vec![i.to_string(), value.to_sat().to_string(), kind.clone()])
        .collect();
    ui::table(&["#", "sat", "type"], &rows);

    ui::heading("value");
    ui::row(
        "inputs",
        report
            .input_total
            .map(ui::sats)
            .unwrap_or_else(|| "unknown".into()),
    );
    ui::row("outputs", ui::sats(report.output_total));
    ui::row(
        "fee",
        report.fee.map(ui::sats).unwrap_or_else(|| "unknown".into()),
    );

    let all_valid = report
        .inputs
        .iter()
        .all(|i| matches!(i.signature_valid, Some(true)));
    if all_valid && report.declared_txid == report.computed_txid {
        ui::note(
            "Every signature was recomputed from the decoded bytes and verified against\n\
             the public key committed to by the previous output. No wallet state involved.",
        );
    }
    Ok(())
}

pub fn compare(cfg: &Config) -> Result<()> {
    let mnemonic = cfg.require_mnemonic()?;

    ui::heading("wpkh vs tr, same seed");
    ui::note(
        "Both descriptors below come from the same mnemonic. They differ only in the\n\
         BIP43 purpose field and the script type, which is enough to make them\n\
         completely separate wallets with separate address sets.",
    );

    let mut rows = Vec::new();
    for kind in [DescriptorKind::Wpkh, DescriptorKind::Tr] {
        let descriptors = keys::descriptors(
            mnemonic,
            cfg.passphrase(),
            cfg.network,
            cfg.coin_type(),
            kind,
        )?;

        // An in-memory wallet is the cheapest way to turn descriptor strings
        // into addresses without creating a second database.
        let wallet = Wallet::create(descriptors.external, descriptors.internal)
            .network(cfg.network)
            .create_wallet_no_persist()?;

        let first = wallet.peek_address(KeychainKind::External, 0);
        let spk = first.address.script_pubkey();
        let public = wallet.public_descriptor(KeychainKind::External).to_string();

        // What it costs to spend one of these outputs, from miniscript's own
        // satisfaction analysis rather than from a rule of thumb.
        let satisfaction = wallet
            .public_descriptor(KeychainKind::External)
            .max_weight_to_satisfy()
            .map_err(|e| Error::wallet("computing the max satisfaction weight", e))?;

        ui::heading(&format!("{} ({})", kind.as_str(), kind.bip()));
        ui::row("account path", &descriptors.account_path);
        ui::block("descriptor (public)", public);
        ui::row("address 0", &first.address);
        ui::row("script pubkey", spk.to_hex_string());
        ui::row("output size", format!("{} B", spk.len() + 9));
        ui::row(
            "spend witness",
            format!("{} wu (max)", satisfaction.to_wu()),
        );

        rows.push(vec![
            kind.as_str().to_string(),
            descriptors.account_path.to_string(),
            (spk.len() + 9).to_string(),
            satisfaction.to_wu().to_string(),
            first.address.to_string().len().to_string(),
        ]);
    }

    ui::heading("summary");
    ui::table(
        &[
            "descriptor",
            "account path",
            "output bytes",
            "witness wu",
            "address chars",
        ],
        &rows,
    );
    ui::note(
        "Taproot pays 12 more bytes per output (a 32-byte x-only key against a 20-byte\n\
         hash) and gets them back on the spend: a key spend is one 64-byte Schnorr\n\
         signature, where wpkh needs a ~72-byte DER signature plus a 33-byte public key.\n\
         The witness column above is miniscript's own worst-case satisfaction weight.",
    );
    Ok(())
}

pub fn new_mnemonic() -> Result<()> {
    let mnemonic = keys::generate_mnemonic()?;
    ui::heading("new mnemonic");
    ui::note(mnemonic.to_string());
    ui::note(
        "Put this in your .env as RFB_MNEMONIC. This is test-network key material:\n\
         never reuse it for anything holding real value.",
    );
    Ok(())
}

fn print_descriptors(handle: &WalletHandle) {
    ui::heading("descriptors (public, as persisted)");
    for keychain in [KeychainKind::External, KeychainKind::Internal] {
        ui::block(
            keychain_name(keychain),
            handle.wallet.public_descriptor(keychain),
        );
    }
}

/// Rows shown by `utxos` before it starts truncating.
const DEFAULT_UTXO_ROWS: usize = 15;

/// Coinbase maturity, in confirmations.
const COINBASE_MATURITY: u32 = 100;

/// Confirmed is not the same as spendable: a coinbase output stays locked for
/// 100 blocks, and the balance command reports it separately.
fn utxo_status(handle: &WalletHandle, utxo: &bdk_wallet::LocalOutput, tip: u32) -> String {
    let ChainPosition::Confirmed { anchor, .. } = &utxo.chain_position else {
        return "unconfirmed".to_string();
    };

    let height = anchor.block_id.height;
    let is_coinbase = handle
        .wallet
        .get_tx(utxo.outpoint.txid)
        .map(|tx| tx.tx_node.tx.is_coinbase())
        .unwrap_or(false);

    if is_coinbase {
        let confirmations = tip.saturating_sub(height) + 1;
        if confirmations < COINBASE_MATURITY {
            return format!(
                "immature @ {height} ({}/{COINBASE_MATURITY})",
                confirmations
            );
        }
    }

    format!("confirmed @ {height}")
}

fn keychain_name(keychain: KeychainKind) -> &'static str {
    match keychain {
        KeychainKind::External => "external (receive)",
        KeychainKind::Internal => "internal (change)",
    }
}

fn position(pos: &ChainPosition<bdk_wallet::chain::ConfirmationBlockTime>) -> String {
    match pos {
        ChainPosition::Confirmed { anchor, .. } => {
            format!("confirmed @ {}", anchor.block_id.height)
        }
        ChainPosition::Unconfirmed { .. } => "unconfirmed".to_string(),
    }
}
