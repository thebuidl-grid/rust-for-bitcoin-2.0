//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use std::str::FromStr;

use bitcoin::{Address, AddressType, Network};

use crate::model::{AddressFormat, AddressReport};
use crate::{LabError, LabResult};

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    if address.starts_with("bc1p") || address.starts_with("tb1p") || address.starts_with("bcrt1p") {
        AddressFormat::P2tr
    } else if address.starts_with("bc1q")
        || address.starts_with("tb1q")
        || address.starts_with("bcrt1q")
    {
        AddressFormat::P2wpkh
    } else if address.starts_with('3') || address.starts_with('2') {
        AddressFormat::P2sh
    } else if address.starts_with('1') || address.starts_with('m') || address.starts_with('n') {
        AddressFormat::P2pkh
    } else {
        AddressFormat::Unknown
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

fn address_format(address: &Address) -> AddressFormat {
    match address.address_type() {
        Some(AddressType::P2pkh) => AddressFormat::P2pkh,
        Some(AddressType::P2sh) => AddressFormat::P2sh,
        Some(AddressType::P2wpkh) => AddressFormat::P2wpkh,
        Some(AddressType::P2tr) => AddressFormat::P2tr,
        _ => AddressFormat::Unknown,
    }
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    let parsed = Address::from_str(address)
        .map_err(|error| LabError::InvalidAddress(error.to_string()))?
        .require_network(network)
        .map_err(|error| LabError::WrongNetwork(error.to_string()))?;

    Ok(AddressReport {
        address: parsed.to_string(),
        network: format!("{network:?}").to_lowercase(),
        format: address_format(&parsed),
        script_pubkey_hex: parsed.script_pubkey().to_hex_string(),
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    Ok(inspect_address(address, network)?.script_pubkey_hex)
}
