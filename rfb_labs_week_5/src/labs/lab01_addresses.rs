//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use bitcoin::address::NetworkUnchecked;
use bitcoin::{Address, Network};

use crate::model::{AddressFormat, AddressReport};
use crate::{LabError, LabResult};

/// Identify an address family from its human-readable prefix.
///
/// This is a *heuristic*: the prefix tells us which encoding/version bytes were used,
/// but it does not prove the address is valid. [`inspect_address`] performs the real
/// checksum and network validation via `rust-bitcoin`.
pub fn identify_prefix(address: &str) -> AddressFormat {
    if address.starts_with("bc1p") || address.starts_with("tb1p") || address.starts_with("bcrt1p") {
        return AddressFormat::P2tr;
    }
    if address.starts_with("bc1q") || address.starts_with("tb1q") || address.starts_with("bcrt1q") {
        return AddressFormat::P2wpkh;
    }

    match address.chars().next() {
        Some('1') | Some('m') | Some('n') => AddressFormat::P2pkh,
        Some('2') | Some('3') => AddressFormat::P2sh,
        _ => AddressFormat::Unknown,
    }
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    let is_mainnet = network == Network::Bitcoin;
    match format {
        AddressFormat::P2pkh => Some(if is_mainnet { "1" } else { "m/n" }),
        AddressFormat::P2sh => Some(if is_mainnet { "3" } else { "2" }),
        AddressFormat::P2wpkh => Some(match network {
            Network::Bitcoin => "bc1q",
            Network::Regtest => "bcrt1q",
            _ => "tb1q",
        }),
        AddressFormat::P2tr => Some(match network {
            Network::Bitcoin => "bc1p",
            Network::Regtest => "bcrt1p",
            _ => "tb1p",
        }),
        AddressFormat::Unknown => None,
    }
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    let unchecked = address
        .parse::<Address<NetworkUnchecked>>()
        .map_err(|error| LabError::InvalidAddress(error.to_string()))?;

    let checked = unchecked
        .require_network(network)
        .map_err(|error| LabError::WrongNetwork(error.to_string()))?;

    let format = match checked.address_type() {
        Some(bitcoin::AddressType::P2pkh) => AddressFormat::P2pkh,
        Some(bitcoin::AddressType::P2sh) => AddressFormat::P2sh,
        Some(bitcoin::AddressType::P2wpkh) => AddressFormat::P2wpkh,
        Some(bitcoin::AddressType::P2tr) => AddressFormat::P2tr,
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
    Ok(inspect_address(address, network)?.script_pubkey_hex)
}
