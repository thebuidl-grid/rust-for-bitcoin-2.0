//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use std::str::FromStr;

use bitcoin::address::{Address, NetworkUnchecked};
use bitcoin::{AddressType, Network};

use crate::error::LabError;
use crate::model::{AddressFormat, AddressReport};
use crate::LabResult;

/// Strip a known bech32 human-readable prefix and return the data part.
fn bech32_body(address: &str) -> Option<&str> {
    for hrp in ["bcrt1", "bc1", "tb1", "sb1"] {
        if let Some(rest) = address.strip_prefix(hrp) {
            return Some(rest);
        }
    }
    None
}

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    let address = address.trim();

    if let Some(body) = bech32_body(&address.to_lowercase()) {
        // The character after the `1` separator encodes the witness version:
        // `q` is version 0 (P2WPKH/P2WSH), `p` is version 1 (P2TR).
        return match body.chars().next() {
            Some('q') => AddressFormat::P2wpkh,
            Some('p') => AddressFormat::P2tr,
            _ => AddressFormat::Unknown,
        };
    }

    match address.chars().next() {
        Some('1') | Some('m') | Some('n') => AddressFormat::P2pkh,
        Some('2') | Some('3') => AddressFormat::P2sh,
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

/// Human-readable network label used in reports.
fn network_label(network: Network) -> &'static str {
    match network {
        Network::Bitcoin => "bitcoin",
        Network::Testnet => "testnet",
        Network::Signet => "signet",
        Network::Regtest => "regtest",
        // `Network` is `#[non_exhaustive]`.
        _ => "unknown",
    }
}

/// Map a parsed address type to the lab's `AddressFormat` model.
fn format_of(address: &Address) -> AddressFormat {
    match address.address_type() {
        Some(AddressType::P2pkh) => AddressFormat::P2pkh,
        Some(AddressType::P2sh) => AddressFormat::P2sh,
        Some(AddressType::P2wpkh) => AddressFormat::P2wpkh,
        Some(AddressType::P2tr) => AddressFormat::P2tr,
        _ => AddressFormat::Unknown,
    }
}

/// Parse and require the correct network, returning a checked address.
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
    Ok(AddressReport {
        address: checked.to_string(),
        network: network_label(network).to_owned(),
        format: format_of(&checked),
        script_pubkey_hex: checked.script_pubkey().to_hex_string(),
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    Ok(checked_address(address, network)?
        .script_pubkey()
        .to_hex_string())
}
