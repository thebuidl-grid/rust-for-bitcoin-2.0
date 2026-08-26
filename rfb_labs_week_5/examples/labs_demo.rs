//! Runs every Week 5 lab function once against public/disposable test data and prints
//! the results. This is the evidence source for `submissions/lab_01.md` .. `lab_10.md`.
//!
//! Every secret key below is a fixed, throwaway `[byte; 32]` array used only to produce
//! a deterministic demo public key — never a real private key. The only mnemonic used
//! anywhere is the published BIP39 test mnemonic. Nothing here should ever hold funds.
//!
//! Run with: `cargo run --example labs_demo`

use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::{Network, PublicKey};

use rfb_labs_week_5::labs::{
    lab01_addresses, lab02_p2pkh, lab03_p2sh, lab04_p2wpkh, lab05_compatibility, lab06_weight_fees,
    lab07_bip39, lab08_bip32, lab09_bip44, lab10_recovery,
};
use rfb_labs_week_5::model::{AddressFormat, SenderCapabilities};

const MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn demo_public_key(byte: u8) -> PublicKey {
    let secp = Secp256k1::new();
    let secret = SecretKey::from_slice(&[byte; 32]).expect("disposable demo key");
    PublicKey::new(secret.public_key(&secp))
}

fn main() {
    lab01();
    lab02();
    lab03();
    lab04();
    lab05();
    lab06();
    lab07();
    lab08();
    lab09();
    lab10();
}

fn lab01() {
    println!("=== Lab 01 — address prefixes, networks, and scriptPubKeys ===");

    for address in [
        "1BoatSLRHtKNngkdXEeobR76b53LETtpyT",
        "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy",
        "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080",
        "bc1pexample",
    ] {
        println!(
            "identify_prefix({address:?}) = {:?}",
            lab01_addresses::identify_prefix(address)
        );
    }

    for format in [
        AddressFormat::P2pkh,
        AddressFormat::P2sh,
        AddressFormat::P2wpkh,
        AddressFormat::P2tr,
    ] {
        println!(
            "expected_prefix({format:?}, Regtest) = {:?}",
            lab01_addresses::expected_prefix(format, Network::Regtest)
        );
    }

    let public = demo_public_key(1);
    let address = bitcoin::Address::p2pkh(public, Network::Regtest);
    let report = lab01_addresses::inspect_address(&address.to_string(), Network::Regtest)
        .expect("valid regtest address");
    println!("inspect_address({address}, Regtest) = {report:?}");

    let wrong_network = lab01_addresses::inspect_address(&address.to_string(), Network::Bitcoin);
    println!("inspect_address({address}, Bitcoin) = {wrong_network:?} (expected: Err)");
    println!();
}

fn lab02() {
    println!("=== Lab 02 — legacy P2PKH lock and ScriptSig ===");

    let public = demo_public_key(2);
    let public_hex = public.to_string();

    println!(
        "derive_p2pkh_address = {:?}",
        lab02_p2pkh::derive_p2pkh_address(&public_hex, Network::Bitcoin)
    );
    println!(
        "build_p2pkh_script_pubkey = {:?}",
        lab02_p2pkh::build_p2pkh_script_pubkey(&public_hex)
    );
    println!(
        "committed_pubkey_hash = {:?}",
        lab02_p2pkh::committed_pubkey_hash(&public_hex)
    );
    println!(
        "p2pkh_spend_template = {:?}",
        lab02_p2pkh::p2pkh_spend_template("30440220deadbeef01", &public_hex)
    );
    println!();
}

fn lab03() {
    println!("=== Lab 03 — P2SH 2-of-3 multisig ===");

    let keys = [demo_public_key(1), demo_public_key(2), demo_public_key(3)];
    let key_strings = keys.map(|key| key.to_string());
    let key_refs = [
        key_strings[0].as_str(),
        key_strings[1].as_str(),
        key_strings[2].as_str(),
    ];

    let report =
        lab03_p2sh::inspect_p2sh_multisig(key_refs, Network::Regtest).expect("valid multisig");
    println!("inspect_p2sh_multisig = {report:?}");
    println!();
}

fn lab04() {
    println!("=== Lab 04 — native P2WPKH ===");

    let public = demo_public_key(4);
    let public_hex = public.to_string();

    println!(
        "derive_p2wpkh_address = {:?}",
        lab04_p2wpkh::derive_p2wpkh_address(&public_hex, Network::Regtest)
    );
    println!(
        "build_p2wpkh_script_pubkey = {:?}",
        lab04_p2wpkh::build_p2wpkh_script_pubkey(&public_hex)
    );
    println!(
        "witness_program = {:?}",
        lab04_p2wpkh::witness_program(&public_hex)
    );
    println!(
        "native_spend_template = {:?}",
        lab04_p2wpkh::native_spend_template("30440220cafebabe01", &public_hex)
    );
    println!();
}

fn lab05() {
    println!("=== Lab 05 — compatibility map ===");

    let p2sh_era = SenderCapabilities {
        base58_p2pkh: true,
        base58_p2sh: true,
        bech32: false,
        bech32m: false,
    };
    let modern = SenderCapabilities {
        base58_p2pkh: true,
        base58_p2sh: true,
        bech32: true,
        bech32m: true,
    };

    println!(
        "compatibility_report(p2sh_era) = {:?}",
        lab05_compatibility::compatibility_report(p2sh_era)
    );
    println!(
        "best_supported_format(p2sh_era) = {:?}",
        lab05_compatibility::best_supported_format(p2sh_era)
    );
    println!(
        "compatibility_report(modern) = {:?}",
        lab05_compatibility::compatibility_report(modern)
    );
    println!(
        "best_supported_format(modern) = {:?}",
        lab05_compatibility::best_supported_format(modern)
    );
    for format in [
        AddressFormat::P2pkh,
        AddressFormat::P2sh,
        AddressFormat::P2wpkh,
        AddressFormat::P2tr,
    ] {
        println!(
            "required_encoding({format:?}) = {:?}",
            lab05_compatibility::required_encoding(format)
        );
    }
    println!();
}

fn lab06() {
    println!("=== Lab 06 — weight, virtual size, and fees ===");

    let weight = lab06_weight_fees::transaction_weight(100, 200).unwrap();
    println!("transaction_weight(100, 200) = {weight}");
    println!(
        "virtual_size(564) = {}",
        lab06_weight_fees::virtual_size(564)
    );
    println!(
        "virtual_size(565) = {}",
        lab06_weight_fees::virtual_size(565)
    );
    println!(
        "fee_sats(141, 50) = {:?}",
        lab06_weight_fees::fee_sats(141, 50)
    );
    println!(
        "compare_fees(226, 141, 50) = {:?}",
        lab06_weight_fees::compare_fees(226, 141, 50)
    );
    println!();
}

fn lab07() {
    println!("=== Lab 07 — BIP39 mnemonic and seed ===");

    println!(
        "inspect_mnemonic(PUBLIC_TEST_MNEMONIC) = {:?}",
        lab07_bip39::inspect_mnemonic(MNEMONIC)
    );
    println!(
        "is_public_test_mnemonic = {}",
        lab07_bip39::is_public_test_mnemonic(MNEMONIC)
    );
    println!(
        "mnemonic_seed_hex(MNEMONIC, \"TREZOR\") = {:?}",
        lab07_bip39::mnemonic_seed_hex(MNEMONIC, "TREZOR")
    );
    let comparison = lab07_bip39::compare_passphrases(MNEMONIC, "class").unwrap();
    println!("compare_passphrases(MNEMONIC, \"class\") = {comparison:?}");
    println!();
}

fn lab08() {
    println!("=== Lab 08 — BIP32 extended keys ===");

    println!(
        "master_xpriv(regtest) = {:?}",
        lab08_bip32::master_xpriv(MNEMONIC, "", Network::Regtest)
    );
    let extended =
        lab08_bip32::derive_extended_keys(MNEMONIC, "", "m/84'/1'/0'", Network::Regtest).unwrap();
    println!("derive_extended_keys(m/84'/1'/0') = {extended:?}");

    let parent_account =
        lab08_bip32::derive_extended_keys(MNEMONIC, "", "m/84'/1'/0'/0", Network::Regtest).unwrap();
    println!(
        "derive_normal_child_xpub(parent.xpub, 7) = {:?}",
        lab08_bip32::derive_normal_child_xpub(&parent_account.xpub, 7)
    );

    println!(
        "path_contains_hardened_step(m/44'/0'/0'/0/0) = {:?}",
        lab08_bip32::path_contains_hardened_step("m/44'/0'/0'/0/0")
    );
    println!(
        "path_contains_hardened_step(m/0/1/2) = {:?}",
        lab08_bip32::path_contains_hardened_step("m/0/1/2")
    );
    println!();
}

fn lab09() {
    println!("=== Lab 09 — BIP44 path decoding ===");

    let info = lab09_bip44::decode_bip44_path("m/44'/0'/2'/1/5").unwrap();
    println!("decode_bip44_path(m/44'/0'/2'/1/5) = {info:?}");
    println!(
        "describe_bip44_path = {:?}",
        lab09_bip44::describe_bip44_path(&info)
    );
    println!(
        "with_address_index(..., 6) = {:?}",
        lab09_bip44::with_address_index("m/44'/0'/2'/1/5", 6)
    );
    println!(
        "derive_bip44_address(m/44'/1'/0'/0/0, Regtest) = {:?}",
        lab09_bip44::derive_bip44_address(MNEMONIC, "", "m/44'/1'/0'/0/0", Network::Regtest)
    );
    println!();
}

fn lab10() {
    println!("=== Lab 10 — deterministic recovery across BIP44/49/84 ===");

    let set = lab10_recovery::derive_address_set(MNEMONIC, "", 0, 0, Network::Regtest).unwrap();
    println!("derive_address_set(account=0, index=0, Regtest) = {set:?}");

    let repeatable = lab10_recovery::recovery_is_repeatable(
        MNEMONIC,
        "class",
        "m/84'/1'/0'/0/0",
        AddressFormat::P2wpkh,
        Network::Regtest,
    )
    .unwrap();
    println!("recovery_is_repeatable(index 0, passphrase \"class\") = {repeatable}");

    let index_changes = lab10_recovery::changing_index_changes_address(
        MNEMONIC,
        "",
        "m/84'/1'/0'/0/0",
        "m/84'/1'/0'/0/1",
        AddressFormat::P2wpkh,
        Network::Regtest,
    )
    .unwrap();
    println!("changing_index_changes_address(index 0 vs 1) = {index_changes}");

    let p2pkh = lab10_recovery::derive_address_for_path(
        MNEMONIC,
        "",
        "m/44'/1'/0'/0/0",
        AddressFormat::P2pkh,
        Network::Regtest,
    )
    .unwrap();
    let p2wpkh = lab10_recovery::derive_address_for_path(
        MNEMONIC,
        "",
        "m/44'/1'/0'/0/0",
        AddressFormat::P2wpkh,
        Network::Regtest,
    )
    .unwrap();
    println!("same path (m/44'/1'/0'/0/0), format P2PKH  = {p2pkh}");
    println!("same path (m/44'/1'/0'/0/0), format P2WPKH = {p2wpkh}");
}
