//! Week 5 evidence runner.
//!
//! Prints the real values produced by every lab so the submission files can quote
//! actual output instead of retyped numbers.
//!
//! ```text
//! cargo run              # every lab
//! cargo run -- 7         # one lab
//! ```
//!
//! Every secret in this program comes from the published BIP39 test mnemonic, which
//! must never receive real funds. Extended private keys are masked before printing.

use bitcoin::secp256k1::{Secp256k1, SecretKey};
use bitcoin::{Address, CompressedPublicKey, Network, PublicKey};

use rfb_labs_week_5::labs::lab01_addresses::{
    expected_prefix, identify_prefix, inspect_address, script_pubkey_hex,
};
use rfb_labs_week_5::labs::lab02_p2pkh::{
    build_p2pkh_script_pubkey, committed_pubkey_hash, derive_p2pkh_address, p2pkh_spend_template,
};
use rfb_labs_week_5::labs::lab03_p2sh::inspect_p2sh_multisig;
use rfb_labs_week_5::labs::lab04_p2wpkh::{
    build_p2wpkh_script_pubkey, derive_p2wpkh_address, native_spend_template, witness_program,
};
use rfb_labs_week_5::labs::lab05_compatibility::{
    best_supported_format, compatibility_report, required_encoding,
};
use rfb_labs_week_5::labs::lab06_weight_fees::{
    compare_fees, fee_sats, transaction_weight, virtual_size,
};
use rfb_labs_week_5::labs::lab07_bip39::{
    compare_passphrases, inspect_mnemonic, is_public_test_mnemonic, mnemonic_seed_hex,
    PUBLIC_TEST_MNEMONIC,
};
use rfb_labs_week_5::labs::lab08_bip32::{
    derive_extended_keys, derive_normal_child_xpub, master_xpriv, path_contains_hardened_step,
};
use rfb_labs_week_5::labs::lab09_bip44::{
    decode_bip44_path, derive_bip44_address, describe_bip44_path, with_address_index,
};
use rfb_labs_week_5::labs::lab10_recovery::{
    changing_index_changes_address, derive_address_for_path, derive_address_set,
    recovery_is_repeatable,
};
use rfb_labs_week_5::model::{AddressFormat, SenderCapabilities};
use rfb_labs_week_5::LabResult;

/// Every lab runs on regtest so no output can be confused with a mainnet address.
const NETWORK: Network = Network::Regtest;

/// Disposable passphrase used only to show that it selects a different wallet.
const CLASS_PASSPHRASE: &str = "class";

/// One entry in the lab menu: number, title, and the function that prints it.
type Lab = (u32, &'static str, fn() -> LabResult<()>);

fn main() {
    let selected: Option<u32> = std::env::args().nth(1).and_then(|value| value.parse().ok());

    let labs: [Lab; 10] = [
        (1, "Address formats and networks", lab01),
        (2, "Legacy P2PKH", lab02),
        (3, "P2SH 2-of-3 multisig", lab03),
        (4, "Native P2WPKH", lab04),
        (5, "Sender compatibility", lab05),
        (6, "Weight, virtual size, and fees", lab06),
        (7, "BIP39 mnemonics and seeds", lab07),
        (8, "BIP32 extended keys", lab08),
        (9, "BIP44 path decoding", lab09),
        (10, "Deterministic recovery", lab10),
    ];

    for (number, title, run) in labs {
        if selected.is_some_and(|wanted| wanted != number) {
            continue;
        }

        println!("== Lab {number:02}: {title} ==");
        if let Err(error) = run() {
            println!("  failed: {error}");
        }
        println!();
    }
}

/// Build a deterministic public key so every run prints the same evidence.
fn demo_public_key(seed_byte: u8) -> PublicKey {
    let secp = Secp256k1::new();
    let secret = SecretKey::from_slice(&[seed_byte; 32]).expect("fixed 32-byte test secret");

    PublicKey::new(secret.public_key(&secp))
}

/// Hide the body of an extended private key while keeping its version prefix visible.
fn mask_xpriv(xpriv: &str) -> String {
    match xpriv.len() > 12 {
        true => format!("{}...{}", &xpriv[..8], &xpriv[xpriv.len() - 4..]),
        false => "<masked>".to_owned(),
    }
}

fn lab01() -> LabResult<()> {
    let secp = Secp256k1::new();
    let public = demo_public_key(1);
    let compressed = CompressedPublicKey::try_from(public).expect("compressed test key");

    let samples = [
        Address::p2pkh(public, NETWORK).to_string(),
        Address::p2sh(&Address::p2pkh(public, NETWORK).script_pubkey(), NETWORK)
            .expect("redeem script under 520 bytes")
            .to_string(),
        Address::p2wpkh(&compressed, NETWORK).to_string(),
        Address::p2tr(&secp, public.inner.x_only_public_key().0, None, NETWORK).to_string(),
    ];

    for address in &samples {
        let report = inspect_address(address, NETWORK)?;
        println!("  {address}");
        println!(
            "    prefix guess {:?} | parsed {:?} | network {}",
            identify_prefix(address),
            report.format,
            report.network
        );
        println!(
            "    expected prefix {:?} | scriptPubKey {}",
            expected_prefix(report.format, NETWORK).unwrap_or("n/a"),
            report.script_pubkey_hex
        );
    }

    // A mainnet address is well formed and still refused on regtest.
    let mainnet = "1BoatSLRHtKNngkdXEeobR76b53LETtpyT";
    println!("  {mainnet}");
    println!(
        "    prefix guess {:?} | regtest check {}",
        identify_prefix(mainnet),
        match script_pubkey_hex(mainnet, NETWORK) {
            Ok(script) => script,
            Err(error) => format!("rejected: {error}"),
        }
    );

    Ok(())
}

fn lab02() -> LabResult<()> {
    let public_key = demo_public_key(2).to_string();
    let signature = "30440220deadbeef01";
    let spend = p2pkh_spend_template(signature, &public_key)?;

    println!("  public key    {public_key}");
    println!("  HASH160       {}", committed_pubkey_hash(&public_key)?);
    println!(
        "  address       {}",
        derive_p2pkh_address(&public_key, NETWORK)?
    );
    println!(
        "  scriptPubKey  {}",
        build_p2pkh_script_pubkey(&public_key)?
    );
    println!("  ScriptSig     {:?}", spend.script_sig_items);
    println!("  witness       {:?}", spend.witness_items);

    Ok(())
}

fn lab03() -> LabResult<()> {
    let keys = [
        demo_public_key(1).to_string(),
        demo_public_key(2).to_string(),
        demo_public_key(3).to_string(),
    ];
    let report = inspect_p2sh_multisig([&keys[0], &keys[1], &keys[2]], NETWORK)?;

    println!("  redeemScript  {}", report.redeem_script_hex);
    println!("  address       {}", report.address);
    println!("  scriptPubKey  {}", report.script_pubkey_hex);
    println!(
        "  redeemScript size {} bytes",
        report.redeem_script_hex.len() / 2
    );

    Ok(())
}

fn lab04() -> LabResult<()> {
    let public_key = demo_public_key(4).to_string();
    let program = witness_program(&public_key)?;
    let spend = native_spend_template("30440220cafebabe01", &public_key)?;

    println!("  public key    {public_key}");
    println!(
        "  address       {}",
        derive_p2wpkh_address(&public_key, NETWORK)?
    );
    println!(
        "  scriptPubKey  {}",
        build_p2wpkh_script_pubkey(&public_key)?
    );
    println!(
        "  witness program v{} {} ({} bytes)",
        program.version, program.program_hex, program.program_length
    );
    println!("  ScriptSig     {:?} (empty)", spend.script_sig_hex);
    println!("  witness       {:?}", spend.witness_items);

    Ok(())
}

fn lab05() -> LabResult<()> {
    let wallets = [
        (
            "2013 P2PKH-only wallet",
            SenderCapabilities {
                base58_p2pkh: true,
                base58_p2sh: false,
                bech32: false,
                bech32m: false,
            },
        ),
        (
            "2017 P2SH-era wallet",
            SenderCapabilities {
                base58_p2pkh: true,
                base58_p2sh: true,
                bech32: false,
                bech32m: false,
            },
        ),
        (
            "2019 Bech32 wallet",
            SenderCapabilities {
                base58_p2pkh: true,
                base58_p2sh: true,
                bech32: true,
                bech32m: false,
            },
        ),
        (
            "2023 Bech32m wallet",
            SenderCapabilities {
                base58_p2pkh: true,
                base58_p2sh: true,
                bech32: true,
                bech32m: true,
            },
        ),
    ];

    for (name, capabilities) in wallets {
        let report = compatibility_report(capabilities);
        println!("  {name}");
        println!(
            "    P2PKH {} | P2SH-P2WPKH {} | P2WPKH {} | P2TR {}",
            report.p2pkh, report.p2sh_p2wpkh, report.p2wpkh, report.p2tr
        );
        println!(
            "    best supported {:?}",
            best_supported_format(capabilities)
        );
    }

    for format in [
        AddressFormat::P2pkh,
        AddressFormat::P2sh,
        AddressFormat::P2wpkh,
        AddressFormat::P2tr,
    ] {
        println!("  {format:?} requires {}", required_encoding(format));
    }

    Ok(())
}

fn lab06() -> LabResult<()> {
    // One input, two outputs, spending P2PKH. There is no witness, so the stripped
    // and total sizes are identical: 226 bytes either way.
    let legacy_weight = transaction_weight(226, 226)?;
    // The same shape spending P2WPKH. The 113 non-witness bytes are unchanged. The
    // 109 witness bytes (marker, flag, signature, public key) are charged at a quarter
    // of that rate.
    let segwit_weight = transaction_weight(113, 222)?;

    println!(
        "  legacy 226 stripped / 226 total -> {legacy_weight} wu -> {} vB",
        virtual_size(legacy_weight)
    );
    println!(
        "  segwit 113 stripped / 222 total -> {segwit_weight} wu -> {} vB",
        virtual_size(segwit_weight)
    );

    let report = compare_fees(226, 141, 50)?;
    println!(
        "  at 50 sat/vB: legacy {} vB = {} sats, segwit {} vB = {} sats, saving {} sats",
        report.legacy_vbytes,
        report.legacy_fee_sats,
        report.segwit_vbytes,
        report.segwit_fee_sats,
        report.savings_sats
    );
    println!("  fee_sats(141, 50) = {}", fee_sats(141, 50)?);
    println!(
        "  transaction_weight(201, 200) = {}",
        match transaction_weight(201, 200) {
            Ok(weight) => weight.to_string(),
            Err(error) => format!("rejected: {error}"),
        }
    );

    Ok(())
}

fn lab07() -> LabResult<()> {
    let report = inspect_mnemonic(PUBLIC_TEST_MNEMONIC)?;
    let invalid = PUBLIC_TEST_MNEMONIC.replace("about", "abandon");
    let passphrases = compare_passphrases(PUBLIC_TEST_MNEMONIC, CLASS_PASSPHRASE)?;

    println!(
        "  published test mnemonic recognized: {}",
        is_public_test_mnemonic(PUBLIC_TEST_MNEMONIC)
    );
    println!(
        "  {} words | {} entropy bits | {} checksum bits",
        report.word_count, report.entropy_bits, report.checksum_bits
    );
    println!(
        "  last word swapped to 'abandon': {}",
        match inspect_mnemonic(&invalid) {
            Ok(_) => "accepted".to_owned(),
            Err(error) => format!("rejected: {error}"),
        }
    );
    println!(
        "  seed with TREZOR passphrase (BIP39 vector)\n    {}",
        mnemonic_seed_hex(PUBLIC_TEST_MNEMONIC, "TREZOR")?
    );
    println!(
        "  seed with no passphrase\n    {}",
        passphrases.empty_passphrase_seed_hex
    );
    println!(
        "  seed with \"{CLASS_PASSPHRASE}\" passphrase\n    {}",
        passphrases.protected_seed_hex
    );
    println!("  seeds differ: {}", passphrases.seeds_differ);

    Ok(())
}

fn lab08() -> LabResult<()> {
    let master = master_xpriv(PUBLIC_TEST_MNEMONIC, "", NETWORK)?;
    let account = derive_extended_keys(PUBLIC_TEST_MNEMONIC, "", "m/84'/1'/0'", NETWORK)?;
    let branch = derive_extended_keys(PUBLIC_TEST_MNEMONIC, "", "m/84'/1'/0'/0", NETWORK)?;
    let child = derive_normal_child_xpub(&branch.xpub, 7)?;

    println!("  master xpriv  {} (masked)", mask_xpriv(&master));
    println!("  path          {}", account.derivation_path);
    println!("  account xpriv {} (masked)", mask_xpriv(&account.xpriv));
    println!("  account xpub  {}", account.xpub);
    println!("  branch xpub   {}", branch.xpub);
    println!("  child 7 xpub  {child}");
    println!(
        "  m/84'/1'/0'/0/0 hardened step: {}",
        path_contains_hardened_step("m/84'/1'/0'/0/0")?
    );
    println!(
        "  m/0/1/2 hardened step: {}",
        path_contains_hardened_step("m/0/1/2")?
    );
    println!(
        "  not/a/path: {}",
        match path_contains_hardened_step("not/a/path") {
            Ok(value) => value.to_string(),
            Err(error) => format!("rejected: {error}"),
        }
    );

    Ok(())
}

fn lab09() -> LabResult<()> {
    let path = "m/44'/0'/2'/1/5";
    let info = decode_bip44_path(path)?;

    println!("  path      {path}");
    println!(
        "  decoded   purpose {} coin_type {} account {} change {} index {}",
        info.purpose, info.coin_type, info.account, info.change, info.index
    );
    println!("  meaning   {}", describe_bip44_path(&info));
    println!("  index 5 -> 6: {}", with_address_index(path, 6)?);

    for index in 0..3 {
        let regtest_path = format!("m/44'/1'/0'/0/{index}");
        println!(
            "  {regtest_path} -> {}",
            derive_bip44_address(PUBLIC_TEST_MNEMONIC, "", &regtest_path, NETWORK)?
        );
    }

    println!(
        "  m/44'/1'/0'/1/0 (change) -> {}",
        derive_bip44_address(PUBLIC_TEST_MNEMONIC, "", "m/44'/1'/0'/1/0", NETWORK)?
    );

    Ok(())
}

fn lab10() -> LabResult<()> {
    let first = derive_address_set(PUBLIC_TEST_MNEMONIC, "", 0, 0, NETWORK)?;
    let repeated = derive_address_set(PUBLIC_TEST_MNEMONIC, "", 0, 0, NETWORK)?;
    let next = derive_address_set(PUBLIC_TEST_MNEMONIC, "", 0, 1, NETWORK)?;

    println!("  index 0");
    println!(
        "    BIP44 m/44'/1'/0'/0/0 P2PKH        {}",
        first.bip44_p2pkh
    );
    println!(
        "    BIP49 m/49'/1'/0'/0/0 P2SH-P2WPKH  {}",
        first.bip49_p2sh_p2wpkh
    );
    println!(
        "    BIP84 m/84'/1'/0'/0/0 P2WPKH       {}",
        first.bip84_p2wpkh
    );
    println!("  same inputs reproduce the set: {}", first == repeated);
    println!("  index 1");
    println!("    BIP44 {}", next.bip44_p2pkh);
    println!("    BIP49 {}", next.bip49_p2sh_p2wpkh);
    println!("    BIP84 {}", next.bip84_p2wpkh);
    println!(
        "  repeatable at m/84'/1'/0'/0/0 with passphrase: {}",
        recovery_is_repeatable(
            PUBLIC_TEST_MNEMONIC,
            CLASS_PASSPHRASE,
            "m/84'/1'/0'/0/0",
            AddressFormat::P2wpkh,
            NETWORK
        )?
    );
    println!(
        "  index change alters the address: {}",
        changing_index_changes_address(
            PUBLIC_TEST_MNEMONIC,
            "",
            "m/84'/1'/0'/0/0",
            "m/84'/1'/0'/0/1",
            AddressFormat::P2wpkh,
            NETWORK
        )?
    );
    println!(
        "  same key m/44'/1'/0'/0/0 as P2WPKH instead: {}",
        derive_address_for_path(
            PUBLIC_TEST_MNEMONIC,
            "",
            "m/44'/1'/0'/0/0",
            AddressFormat::P2wpkh,
            NETWORK
        )?
    );
    println!(
        "  same path with the \"{CLASS_PASSPHRASE}\" passphrase: {}",
        derive_address_for_path(
            PUBLIC_TEST_MNEMONIC,
            CLASS_PASSPHRASE,
            "m/84'/1'/0'/0/0",
            AddressFormat::P2wpkh,
            NETWORK
        )?
    );

    Ok(())
}
