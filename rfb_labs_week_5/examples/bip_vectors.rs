//! Cross-check the lab derivations against the published BIP test vectors.
//!
//! Every value here comes from a BIP document and the public `abandon ... about`
//! mnemonic, so nothing in the output is secret.
//!
//! ```text
//! cargo run --example bip_vectors
//! ```

use bitcoin::Network;

use rfb_labs_week_5::labs::lab07_bip39::{mnemonic_seed_hex, PUBLIC_TEST_MNEMONIC};
use rfb_labs_week_5::labs::lab10_recovery::derive_address_for_path;
use rfb_labs_week_5::model::AddressFormat;

fn main() {
    // BIP39 English test vector for the all-`abandon` entropy with passphrase TREZOR.
    check(
        "BIP39 seed (passphrase TREZOR)",
        &mnemonic_seed_hex(PUBLIC_TEST_MNEMONIC, "TREZOR").expect("valid test mnemonic"),
        "c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a698\
         7599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04",
    );

    // BIP44 account 0, first receive address on mainnet.
    check_address(
        "BIP44 m/44'/0'/0'/0/0",
        "m/44'/0'/0'/0/0",
        AddressFormat::P2pkh,
        Network::Bitcoin,
        "1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA",
    );

    // BIP49 test vector, which is specified on testnet and therefore shares its
    // Base58Check version byte with regtest.
    check_address(
        "BIP49 m/49'/1'/0'/0/0",
        "m/49'/1'/0'/0/0",
        AddressFormat::P2sh,
        Network::Testnet,
        "2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2",
    );

    // BIP84 test vector, specified on mainnet with coin type 0'.
    check_address(
        "BIP84 m/84'/0'/0'/0/0",
        "m/84'/0'/0'/0/0",
        AddressFormat::P2wpkh,
        Network::Bitcoin,
        "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu",
    );

    check_address(
        "BIP84 m/84'/0'/0'/0/1",
        "m/84'/0'/0'/0/1",
        AddressFormat::P2wpkh,
        Network::Bitcoin,
        "bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g",
    );
}

fn check_address(label: &str, path: &str, format: AddressFormat, network: Network, expected: &str) {
    let derived = derive_address_for_path(PUBLIC_TEST_MNEMONIC, "", path, format, network)
        .expect("valid path and format");

    check(label, &derived, expected);
}

fn check(label: &str, derived: &str, expected: &str) {
    let expected: String = expected.split_whitespace().collect();
    let status = if derived == expected {
        "MATCH"
    } else {
        "MISMATCH"
    };

    println!("  {status:8} {label}");
    println!("           derived  {derived}");
    if derived != expected {
        println!("           expected {expected}");
    }
}
