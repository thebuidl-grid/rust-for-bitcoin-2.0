//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use bitcoin::address::NetworkUnchecked;
use bitcoin::{Address, Network};
use std::str::FromStr;

use crate::error::LabError;
use crate::model::{AddressFormat, AddressReport};
use crate::LabResult;

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    let lower = address.to_lowercase();
    if lower.starts_with("bc1p") || lower.starts_with("tb1p") || lower.starts_with("bcrt1p") {
        AddressFormat::P2tr
    } else if lower.starts_with("bc1q") || lower.starts_with("tb1q") || lower.starts_with("bcrt1q")
    {
        AddressFormat::P2wpkh
    } else if address.starts_with('1') || address.starts_with('m') || address.starts_with('n') {
        AddressFormat::P2pkh
    } else if address.starts_with('3') || address.starts_with('2') {
        AddressFormat::P2sh
    } else {
        AddressFormat::Unknown
    }
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    match (format, network) {
        (AddressFormat::P2pkh, Network::Bitcoin) => Some("1"),
        (AddressFormat::P2pkh, Network::Regtest | Network::Testnet | Network::Signet) => {
            Some("m/n")
        }
        (AddressFormat::P2sh, Network::Bitcoin) => Some("3"),
        (AddressFormat::P2sh, Network::Regtest | Network::Testnet | Network::Signet) => Some("2"),
        (AddressFormat::P2wpkh, Network::Bitcoin) => Some("bc1q"),
        (AddressFormat::P2wpkh, Network::Testnet | Network::Signet) => Some("tb1q"),
        (AddressFormat::P2wpkh, Network::Regtest) => Some("bcrt1q"),
        (AddressFormat::P2tr, Network::Bitcoin) => Some("bc1p"),
        (AddressFormat::P2tr, Network::Testnet | Network::Signet) => Some("tb1p"),
        (AddressFormat::P2tr, Network::Regtest) => Some("bcrt1p"),
        _ => None,
    }
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address_str: &str, network: Network) -> LabResult<AddressReport> {
    let unchecked: Address<NetworkUnchecked> =
        Address::from_str(address_str).map_err(|e| LabError::InvalidAddress(e.to_string()))?;

    let checked = unchecked
        .require_network(network)
        .map_err(|e| LabError::WrongNetwork(e.to_string()))?;

    let spk = checked.script_pubkey();

    let format = if spk.is_p2pkh() {
        AddressFormat::P2pkh
    } else if spk.is_p2sh() {
        AddressFormat::P2sh
    } else if spk.is_p2wpkh() {
        AddressFormat::P2wpkh
    } else if spk.is_p2tr() {
        AddressFormat::P2tr
    } else {
        AddressFormat::Unknown
    };

    Ok(AddressReport {
        address: checked.to_string(),
        network: network.to_string(),
        format,
        script_pubkey_hex: spk.to_hex_string(),
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    let report = inspect_address(address, network)?;
    Ok(report.script_pubkey_hex)
}
