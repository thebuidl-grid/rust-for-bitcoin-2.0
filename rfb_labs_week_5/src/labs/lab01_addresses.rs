//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use std::str::FromStr;

use bitcoin::{Address, AddressType, Network};

use crate::model::{AddressFormat, AddressReport};
use crate::{LabError, LabResult};

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    if address.starts_with('1') {
        return AddressFormat::P2pkh;
    }
    if address.starts_with('3') || address.starts_with('2') {
        return AddressFormat::P2sh;
    }
    // Bech32/Bech32m addresses are `<hrp>1<data>`; the human-readable part ends
    // at the last '1', and the very next character (the SegWit witness version)
    // tells P2WPKH ('q' == version 0) apart from P2TR ('p' == version 1).
    if let Some(separator) = address.rfind('1') {
        match address[separator + 1..].chars().next() {
            Some('q') => return AddressFormat::P2wpkh,
            Some('p') => return AddressFormat::P2tr,
            _ => {}
        }
    }
    AddressFormat::Unknown
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    match (format, network) {
        (AddressFormat::P2pkh, Network::Bitcoin) => Some("1"),
        (AddressFormat::P2pkh, Network::Regtest | Network::Testnet | Network::Testnet4) => {
            Some("m/n")
        }
        (AddressFormat::P2sh, Network::Bitcoin) => Some("3"),
        (AddressFormat::P2sh, Network::Regtest | Network::Testnet | Network::Testnet4) => Some("2"),
        (AddressFormat::P2wpkh, Network::Bitcoin) => Some("bc1q"),
        (AddressFormat::P2wpkh, Network::Regtest) => Some("bcrt1q"),
        (AddressFormat::P2wpkh, Network::Testnet | Network::Testnet4) => Some("tb1q"),
        (AddressFormat::P2tr, Network::Bitcoin) => Some("bc1p"),
        (AddressFormat::P2tr, Network::Regtest) => Some("bcrt1p"),
        (AddressFormat::P2tr, Network::Testnet | Network::Testnet4) => Some("tb1p"),
        _ => None,
    }
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    let checked = Address::from_str(address)
        .map_err(|error| LabError::InvalidAddress(error.to_string()))?
        .require_network(network)
        .map_err(|error| LabError::WrongNetwork(error.to_string()))?;

    let format = match checked.address_type() {
        Some(AddressType::P2pkh) => AddressFormat::P2pkh,
        Some(AddressType::P2sh) => AddressFormat::P2sh,
        Some(AddressType::P2wpkh) => AddressFormat::P2wpkh,
        Some(AddressType::P2tr) => AddressFormat::P2tr,
        _ => AddressFormat::Unknown,
    };

    Ok(AddressReport {
        address: checked.to_string(),
        network: network.to_string(),
        format,
        script_pubkey_hex: checked.script_pubkey().to_hex_string(),
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    let checked = Address::from_str(address)
        .map_err(|error| LabError::InvalidAddress(error.to_string()))?
        .require_network(network)
        .map_err(|error| LabError::WrongNetwork(error.to_string()))?;

    Ok(checked.script_pubkey().to_hex_string())
}
