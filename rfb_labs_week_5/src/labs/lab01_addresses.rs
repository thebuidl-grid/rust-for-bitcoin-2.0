//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use bitcoin::Network;
use std::str::FromStr;

use crate::error::LabError;
use crate::model::{AddressFormat, AddressReport};
use crate::LabResult;

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    match address {
        // P2PKH: starts with 1 (mainnet), m or n (regtest/testnet)
        s if s.starts_with('1') || s.starts_with('m') || s.starts_with('n') => AddressFormat::P2pkh,
        // P2SH: starts with 3 (mainnet), 2 (regtest/testnet)
        s if s.starts_with('3') || s.starts_with('2') => AddressFormat::P2sh,
        // P2WPKH Bech32: starts with bc1q (mainnet), tb1q (testnet), bcrt1q (regtest)
        s if s.starts_with("bc1q") || s.starts_with("tb1q") || s.starts_with("bcrt1q") => {
            AddressFormat::P2wpkh
        }
        // P2TR Bech32m: starts with bc1p (mainnet), tb1p (testnet), bcrt1p (regtest)
        s if s.starts_with("bc1p") || s.starts_with("tb1p") || s.starts_with("bcrt1p") => {
            AddressFormat::P2tr
        }
        _ => AddressFormat::Unknown,
    }
}

/// Infer network from address string based on its prefix
fn infer_network_from_prefix(address: &str) -> Option<Network> {
    match address {
        // Mainnet prefixes
        s if s.starts_with('1') || s.starts_with('3') => Some(Network::Bitcoin),
        // Testnet prefixes
        s if s.starts_with("tb1") => Some(Network::Testnet),
        // Regtest prefixes
        s if s.starts_with("bcrt1") || s.starts_with("m") || s.starts_with("n") => {
            Some(Network::Regtest)
        }
        // Mainnet Bech32
        s if s.starts_with("bc1") => Some(Network::Bitcoin),
        _ => None,
    }
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    match (format, network) {
        // P2PKH prefixes
        (AddressFormat::P2pkh, Network::Bitcoin) => Some("1"),
        (AddressFormat::P2pkh, Network::Testnet) => Some("m/n"),
        (AddressFormat::P2pkh, Network::Signet) => Some("m/n"),
        (AddressFormat::P2pkh, Network::Regtest) => Some("m/n"),
        (AddressFormat::P2pkh, Network::Testnet4) => Some("m/n"),
        // P2SH prefixes
        (AddressFormat::P2sh, Network::Bitcoin) => Some("3"),
        (AddressFormat::P2sh, Network::Testnet) => Some("2"),
        (AddressFormat::P2sh, Network::Signet) => Some("2"),
        (AddressFormat::P2sh, Network::Regtest) => Some("2"),
        (AddressFormat::P2sh, Network::Testnet4) => Some("2"),
        // P2WPKH prefixes
        (AddressFormat::P2wpkh, Network::Bitcoin) => Some("bc1q"),
        (AddressFormat::P2wpkh, Network::Testnet) => Some("tb1q"),
        (AddressFormat::P2wpkh, Network::Signet) => Some("tb1q"),
        (AddressFormat::P2wpkh, Network::Regtest) => Some("bcrt1q"),
        (AddressFormat::P2wpkh, Network::Testnet4) => Some("tb1q"),
        // P2TR prefixes
        (AddressFormat::P2tr, Network::Bitcoin) => Some("bc1p"),
        (AddressFormat::P2tr, Network::Testnet) => Some("tb1p"),
        (AddressFormat::P2tr, Network::Signet) => Some("tb1p"),
        (AddressFormat::P2tr, Network::Regtest) => Some("bcrt1p"),
        (AddressFormat::P2tr, Network::Testnet4) => Some("tb1p"),
        // Unknown format
        (AddressFormat::Unknown, _) => None,
    }
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    // Parse the address without network checking first
    let unchecked_addr =
        bitcoin::Address::from_str(address).map_err(|e| LabError::InvalidAddress(e.to_string()))?;

    // Get the checked address
    let parsed_address = unchecked_addr.assume_checked_ref();

    // Infer the network from the address prefix
    let inferred_network = infer_network_from_prefix(address)
        .ok_or_else(|| LabError::InvalidAddress("Cannot infer network from address".to_string()))?;

    // Check network
    if inferred_network != network {
        return Err(LabError::WrongNetwork(format!(
            "Address is for {:?}, expected {:?}",
            inferred_network, network
        )));
    }

    // Identify format
    let format = match parsed_address.address_type() {
        Some(bitcoin::address::AddressType::P2pkh) => AddressFormat::P2pkh,
        Some(bitcoin::address::AddressType::P2sh) => AddressFormat::P2sh,
        Some(bitcoin::address::AddressType::P2wpkh) => AddressFormat::P2wpkh,
        Some(bitcoin::address::AddressType::P2wsh) => AddressFormat::Unknown, // Not in our enum
        Some(bitcoin::address::AddressType::P2tr) => AddressFormat::P2tr,
        Some(_) => AddressFormat::Unknown, // Handle any other types
        None => AddressFormat::Unknown,
    };

    Ok(AddressReport {
        address: address.to_string(),
        network: format!("{:?}", network).to_lowercase(),
        format,
        script_pubkey_hex: parsed_address.script_pubkey().to_hex_string(),
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    let report = inspect_address(address, network)?;
    Ok(report.script_pubkey_hex)
}
