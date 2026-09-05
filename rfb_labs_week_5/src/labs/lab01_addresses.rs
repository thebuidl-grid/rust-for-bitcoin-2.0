//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use std::str::FromStr;

use bitcoin::address::{Address, NetworkUnchecked};
use bitcoin::{AddressType, Network};

use crate::model::{AddressFormat, AddressReport};
use crate::{LabError, LabResult};

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    if address.starts_with("bc1q") || address.starts_with("tb1q") || address.starts_with("bcrt1q") {
        AddressFormat::P2wpkh
    } else if address.starts_with("bc1p")
        || address.starts_with("tb1p")
        || address.starts_with("bcrt1p")
    {
        AddressFormat::P2tr
    } else if address.starts_with('1') {
        AddressFormat::P2pkh
    } else if address.starts_with('3') || address.starts_with('2') {
        AddressFormat::P2sh
    } else {
        AddressFormat::Unknown
    }
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    use AddressFormat::*;
    use Network::*;

    match (format, network) {
        (P2pkh, Bitcoin) => Some("1"),
        (P2pkh, Testnet | Testnet4 | Signet | Regtest) => Some("m/n"),

        (P2sh, Bitcoin) => Some("3"),
        (P2sh, Testnet | Testnet4 | Signet | Regtest) => Some("2"),

        (P2wpkh, Bitcoin) => Some("bc1q"),
        (P2wpkh, Testnet | Testnet4 | Signet) => Some("tb1q"),
        (P2wpkh, Regtest) => Some("bcrt1q"),

        (P2tr, Bitcoin) => Some("bc1p"),
        (P2tr, Testnet | Testnet4 | Signet) => Some("tb1p"),
        (P2tr, Regtest) => Some("bcrt1p"),

        (Unknown, _) => None,
    }
}

fn network_name(network: Network) -> &'static str {
    match network {
        Network::Bitcoin => "bitcoin",
        Network::Testnet => "testnet",
        Network::Testnet4 => "testnet4",
        Network::Signet => "signet",
        Network::Regtest => "regtest",
    }
}

fn checked_address(address: &str, network: Network) -> LabResult<Address> {
    let unchecked = Address::<NetworkUnchecked>::from_str(address)
        .map_err(|error| LabError::InvalidAddress(error.to_string()))?;

    unchecked
        .require_network(network)
        .map_err(|error| LabError::WrongNetwork(error.to_string()))
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    let checked = checked_address(address, network)?;

    let format = match checked.address_type() {
        Some(AddressType::P2pkh) => AddressFormat::P2pkh,
        Some(AddressType::P2sh) => AddressFormat::P2sh,
        Some(AddressType::P2wpkh) => AddressFormat::P2wpkh,
        Some(AddressType::P2tr) => AddressFormat::P2tr,
        _ => AddressFormat::Unknown,
    };

    Ok(AddressReport {
        address: checked.to_string(),
        network: network_name(network).to_string(),
        format,
        script_pubkey_hex: checked.script_pubkey().to_hex_string(),
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    let checked = checked_address(address, network)?;
    Ok(checked.script_pubkey().to_hex_string())
}
