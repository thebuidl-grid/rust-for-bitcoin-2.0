//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use std::str::FromStr;

use bitcoin::{Address, Network};

use crate::model::{AddressFormat, AddressReport};
use crate::LabResult;

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    if address.starts_with('1') {
        AddressFormat::P2pkh
    } else if address.starts_with('3') {
        AddressFormat::P2sh
    } else if address.starts_with("bc1q")
        || address.starts_with("tb1q")
        || address.starts_with("bcrt1q")
    {
        AddressFormat::P2wpkh
    } else if address.starts_with("bc1p")
        || address.starts_with("tb1p")
        || address.starts_with("bcrt1p")
    {
        AddressFormat::P2tr
    } else if address.starts_with('m') || address.starts_with('n') {
        AddressFormat::P2pkh
    } else if address.starts_with('2') {
        AddressFormat::P2sh
    } else {
        AddressFormat::Unknown
    }
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    match (format, network) {
        (AddressFormat::P2pkh, Network::Bitcoin) => Some("1"),
        (AddressFormat::P2sh, Network::Bitcoin) => Some("3"),
        (AddressFormat::P2wpkh, Network::Bitcoin) => Some("bc1q"),
        (AddressFormat::P2tr, Network::Bitcoin) => Some("bc1p"),

        (AddressFormat::P2pkh, Network::Testnet) => Some("m/n"),
        (AddressFormat::P2sh, Network::Testnet) => Some("2"),
        (AddressFormat::P2wpkh, Network::Testnet) => Some("tb1q"),
        (AddressFormat::P2tr, Network::Testnet) => Some("tb1p"),

        (AddressFormat::P2pkh, Network::Signet) => Some("m/n"),
        (AddressFormat::P2sh, Network::Signet) => Some("2"),
        (AddressFormat::P2wpkh, Network::Signet) => Some("tb1q"),
        (AddressFormat::P2tr, Network::Signet) => Some("tb1p"),

        (AddressFormat::P2pkh, Network::Regtest) => Some("m/n"),
        (AddressFormat::P2sh, Network::Regtest) => Some("2"),
        (AddressFormat::P2wpkh, Network::Regtest) => Some("bcrt1q"),
        (AddressFormat::P2tr, Network::Regtest) => Some("bcrt1p"),

        _ => None,
    }
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    let unchecked = Address::from_str(address)
        .map_err(|error| crate::LabError::InvalidAddress(error.to_string()))?;

    let checked = unchecked
        .require_network(network)
        .map_err(|error| crate::LabError::WrongNetwork(error.to_string()))?;

    let format = identify_prefix(address);

    if format == AddressFormat::Unknown {
        return Err(crate::LabError::InvalidAddress(
            "unsupported address format".to_string(),
        ));
    }

    Ok(AddressReport {
        address: address.to_string(),
        network: network.to_string(),
        format,
        script_pubkey_hex: checked.script_pubkey().to_hex_string(),
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    let report = inspect_address(address, network)?;
    Ok(report.script_pubkey_hex)
}
