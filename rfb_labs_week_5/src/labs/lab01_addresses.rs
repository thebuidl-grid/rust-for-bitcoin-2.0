//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use bitcoin::Network;
use std::str::FromStr;

use crate::model::{AddressFormat, AddressReport};
use crate::LabResult;

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    match address {
        a if a.starts_with("1") => AddressFormat::P2pkh,
        a if a.starts_with("3") => AddressFormat::P2sh,
        a if a.starts_with("m") || a.starts_with("n") => AddressFormat::P2pkh,
        a if a.starts_with("2") => AddressFormat::P2sh,
        a if a.starts_with("bc1q") => AddressFormat::P2wpkh,
        a if a.starts_with("tb1q") || a.starts_with("bcrt1q") => AddressFormat::P2wpkh,
        a if a.starts_with("bc1p") => AddressFormat::P2tr,
        a if a.starts_with("tb1p") || a.starts_with("bcrt1p") => AddressFormat::P2tr,
        _ => AddressFormat::Unknown,
    }
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    match (format, network) {
        (AddressFormat::P2pkh, Network::Bitcoin) => Some("1"),
        (AddressFormat::P2pkh, _) => Some("m/n"),
        (AddressFormat::P2sh, Network::Bitcoin) => Some("3"),
        (AddressFormat::P2sh, _) => Some("2"),
        (AddressFormat::P2wpkh, Network::Bitcoin) => Some("bc1q"),
        (AddressFormat::P2wpkh, Network::Regtest) => Some("bcrt1q"),
        (AddressFormat::P2wpkh, _) => Some("tb1q"),
        (AddressFormat::P2tr, Network::Bitcoin) => Some("bc1p"),
        (AddressFormat::P2tr, Network::Regtest) => Some("bcrt1p"),
        (AddressFormat::P2tr, _) => Some("tb1p"),
        (AddressFormat::Unknown, _) => None,
    }
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    let parsed = bitcoin::Address::from_str(address)
        .map_err(|e| crate::LabError::InvalidAddress(e.to_string()))?
        .assume_checked();

    let format = identify_prefix(address);
    let expected = expected_prefix(format, network);

    if !prefix_matches(address, expected) && format != AddressFormat::Unknown {
        return Err(crate::LabError::WrongNetwork(
            "address network does not match expected network".to_string(),
        ));
    }

    let script_pubkey_hex = parsed.script_pubkey().to_hex_string();

    Ok(AddressReport {
        address: address.to_string(),
        network: network_to_string(network),
        format,
        script_pubkey_hex,
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    let parsed = bitcoin::Address::from_str(address)
        .map_err(|e| crate::LabError::InvalidAddress(e.to_string()))?
        .assume_checked();

    let format = identify_prefix(address);
    let expected = expected_prefix(format, network);

    if !prefix_matches(address, expected) && format != AddressFormat::Unknown {
        return Err(crate::LabError::WrongNetwork(
            "address network does not match expected network".to_string(),
        ));
    }

    Ok(parsed.script_pubkey().to_hex_string())
}

fn prefix_matches(address: &str, expected: Option<&str>) -> bool {
    match expected {
        None => false,
        Some(prefix) if prefix.contains('/') => {
            let prefixes: Vec<&str> = prefix.split('/').collect();
            prefixes.iter().any(|p| address.starts_with(p))
        }
        Some(prefix) => address.starts_with(prefix),
    }
}

fn network_to_string(network: Network) -> String {
    match network {
        Network::Bitcoin => "bitcoin".to_string(),
        Network::Testnet => "testnet".to_string(),
        Network::Signet => "signet".to_string(),
        Network::Regtest => "regtest".to_string(),
        _ => "unknown".to_string(),
    }
}
