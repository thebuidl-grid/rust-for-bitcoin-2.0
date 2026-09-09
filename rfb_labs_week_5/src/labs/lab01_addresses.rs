//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use crate::model::{AddressFormat, AddressReport};
use crate::LabResult;
use bitcoin::address::NetworkUnchecked;
use bitcoin::Network;
use bitcoin::{Address, AddressType};
use std::str::FromStr;

use crate::error::LabError;

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    let lower = address.to_ascii_lowercase();

    if lower.starts_with("bc1q") || lower.starts_with("tb1q") || lower.starts_with("bcrt1q") {
        AddressFormat::P2wpkh
    } else if lower.starts_with("bc1p") || lower.starts_with("tb1p") || lower.starts_with("bcrt1p")
    {
        AddressFormat::P2tr
    } else if lower.starts_with('1') || lower.starts_with('m') || lower.starts_with('n') {
        AddressFormat::P2pkh
    } else if lower.starts_with('3') || lower.starts_with('2') {
        AddressFormat::P2sh
    } else {
        AddressFormat::Unknown
    }
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    match network {
        Network::Bitcoin => match format {
            AddressFormat::P2pkh => Some("1"),
            AddressFormat::P2sh => Some("3"),
            AddressFormat::P2wpkh => Some("bc1q"),
            AddressFormat::P2tr => Some("bc1p"),
            AddressFormat::Unknown => None,
        },
        Network::Testnet | Network::Signet => match format {
            AddressFormat::P2pkh => Some("m/n"),
            AddressFormat::P2sh => Some("2"),
            AddressFormat::P2wpkh => Some("tb1q"),
            AddressFormat::P2tr => Some("tb1p"),
            AddressFormat::Unknown => None,
        },
        Network::Regtest => match format {
            AddressFormat::P2pkh => Some("m/n"),
            AddressFormat::P2sh => Some("2"),
            AddressFormat::P2wpkh => Some("bcrt1q"),
            AddressFormat::P2tr => Some("bcrt1p"),
            AddressFormat::Unknown => None,
        },
        _ => None,
    }
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    let unchecked: Address<NetworkUnchecked> =
        Address::from_str(address).map_err(|e| LabError::InvalidAddress(e.to_string()))?;
    let checked = unchecked
        .require_network(network)
        .map_err(|e| LabError::WrongNetwork(e.to_string()))?;

    let format = match checked.address_type() {
        Some(AddressType::P2pkh) => AddressFormat::P2pkh,
        Some(AddressType::P2sh) => AddressFormat::P2sh,
        Some(AddressType::P2wpkh) => AddressFormat::P2wpkh,
        Some(AddressType::P2tr) => AddressFormat::P2tr,
        _ => AddressFormat::Unknown,
    };

    Ok(AddressReport {
        address: checked.to_string(),
        network: format!("{network:?}").to_lowercase(),
        format,
        script_pubkey_hex: checked.script_pubkey().to_hex_string(),
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    let unchecked: Address<NetworkUnchecked> =
        Address::from_str(address).map_err(|e| LabError::InvalidAddress(e.to_string()))?;
    let checked = unchecked
        .require_network(network)
        .map_err(|e| LabError::WrongNetwork(e.to_string()))?;

    Ok(checked.script_pubkey().to_hex_string())
}
