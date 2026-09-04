// === Unit-style tests (always run)
//
// These need no node. They cover descriptor derivation and the raw rust-bitcoin
// helpers.

use rfb_labs_week_6::keys::{self, DescriptorKind};
use rfb_labs_week_6::raw_demo;

use bdk_wallet::bitcoin::Network;

#[test]
fn descriptor_derivation_is_deterministic_and_splits_keychains() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    let a =
        keys::derive_descriptors(mnemonic, None, DescriptorKind::Wpkh, Network::Regtest).unwrap();
    let b =
        keys::derive_descriptors(mnemonic, None, DescriptorKind::Wpkh, Network::Regtest).unwrap();

    assert_eq!(a.external, b.external, "derivation must be deterministic");
    assert_ne!(
        a.external, a.internal,
        "external and internal keychains must differ"
    );
    assert!(
        a.external.contains("/84'/1'/0'/0/*)"),
        "wpkh external keychain follows the BIP84 path on the /0 branch: {}",
        a.external
    );
    assert!(
        a.internal.contains("/84'/1'/0'/1/*)"),
        "wpkh internal keychain follows the BIP84 path on the /1 branch: {}",
        a.internal
    );
    assert!(a.external.contains(")#"), "descriptor carries a checksum");
}

#[test]
fn taproot_descriptor_uses_bip86_path() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let d = keys::derive_descriptors(mnemonic, None, DescriptorKind::Tr, Network::Testnet).unwrap();
    assert!(d.external.starts_with("tr("));
    assert!(d.external.contains("/86'/1'/0'/0/*)"), "{}", d.external);
}

#[test]
fn generated_mnemonic_is_twelve_words_and_valid() {
    let phrase = keys::generate_mnemonic().unwrap();
    assert_eq!(phrase.split_whitespace().count(), 12);
    // A generated mnemonic must derive without error.
    keys::derive_descriptors(&phrase, None, DescriptorKind::Wpkh, Network::Regtest).unwrap();
}

#[test]
fn descriptor_kind_parsing() {
    assert_eq!(
        "wpkh".parse::<DescriptorKind>().unwrap(),
        DescriptorKind::Wpkh
    );
    assert_eq!(
        "taproot".parse::<DescriptorKind>().unwrap(),
        DescriptorKind::Tr
    );
    assert!("p2pk".parse::<DescriptorKind>().is_err());
}

#[test]
fn raw_op_return_roundtrips_through_the_decoder() {
    let (txid, hex) = raw_demo::build_op_return("rfb-week6").unwrap();
    let described = raw_demo::decode(&hex).unwrap();
    assert!(described.contains(&txid.to_string()));
    assert!(described.contains("op_return"));
}

#[test]
fn oversized_op_return_is_rejected() {
    let big = "x".repeat(200);
    assert!(raw_demo::build_op_return(&big).is_err());
}

// === End-to-end regtest test (ignored by default)
//
// Run with a regtest node already up:
//
//   RFB_IT_RPC=127.0.0.1:18443 \
//   RFB_IT_COOKIE=/path/to/regtest/.cookie \
//   cargo test --test wallet -- --ignored --nocapture
//
// It creates a wallet in a temp dir, mines to it, sends a payment, then reopens
// the wallet in a fresh process-equivalent (new `Wallet` value) to prove the
// SQLite state survived.

#[test]
#[ignore = "requires a running regtest bitcoind; see the comment above"]
fn end_to_end_regtest() {
    use rfb_labs_week_6::config::{Config, RpcAuth};
    use rfb_labs_week_6::node::Node;
    use rfb_labs_week_6::wallet::Wallet;
    use rfb_labs_week_6::{sync, tx};
    use std::path::PathBuf;

    let rpc_url = std::env::var("RFB_IT_RPC").expect("set RFB_IT_RPC");
    let cookie = std::env::var("RFB_IT_COOKIE").expect("set RFB_IT_COOKIE");

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("wallet.sqlite");

    let config = Config {
        network: Network::Regtest,
        rpc_url,
        rpc_auth: RpcAuth::Cookie(PathBuf::from(cookie)),
        mnemonic: keys::generate_mnemonic().unwrap(),
        passphrase: None,
        descriptor_kind: DescriptorKind::Wpkh,
        db_path: db_path.clone(),
        start_height: 0,
    };

    let node = Node::connect(&config).expect("connect to regtest");

    let mut wallet = Wallet::create(&config).unwrap();
    let address = wallet.new_receive_address().unwrap().address;

    node.mine(101, &address).unwrap();
    sync::run(&mut wallet, &node, 0).unwrap();
    assert!(wallet.balance().trusted_spendable().to_sat() > 0);

    // Send back to a wallet-owned address (self-send keeps the test self-contained).
    let dest = wallet.new_receive_address().unwrap().address;
    let request = tx::SpendRequest {
        recipient: dest,
        amount: bdk_wallet::bitcoin::Amount::from_sat(1_000_000),
        fee_rate: None,
        selected_utxos: vec![],
        drain: false,
    };
    let outcome = tx::send(&mut wallet, &node, request).unwrap();
    assert!(!outcome.raw_hex.is_empty());

    node.mine(1, &address).unwrap();

    // Reopen from disk: no re-derive of chain state, just a load.
    drop(wallet);
    let reopened = Wallet::open(&config).unwrap();
    assert!(
        reopened.utxos().iter().any(|u| u.confirmed),
        "confirmed UTXOs must survive a reopen"
    );
}
