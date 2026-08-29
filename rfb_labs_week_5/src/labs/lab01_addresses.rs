//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use bitcoin::address::NetworkUnchecked;
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

    let lower = address.to_ascii_lowercase();
    for hrp in ["bc", "tb", "bcrt"] {
        let prefix = format!("{hrp}1");
        if let Some(rest) = lower.strip_prefix(prefix.as_str()) {
            return match rest.chars().next() {
                Some('q') => AddressFormat::P2wpkh,
                Some('p') => AddressFormat::P2tr,
                _ => AddressFormat::Unknown,
            };
        }
    }

    AddressFormat::Unknown
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    match format {
        AddressFormat::P2pkh => match network {
            Network::Bitcoin => Some("1"),
            _ => Some("m/n"),
        },
        AddressFormat::P2sh => match network {
            Network::Bitcoin => Some("3"),
            _ => Some("2"),
        },
        AddressFormat::P2wpkh => match network {
            Network::Bitcoin => Some("bc1q"),
            Network::Regtest => Some("bcrt1q"),
            _ => Some("tb1q"),
        },
        AddressFormat::P2tr => match network {
            Network::Bitcoin => Some("bc1p"),
            Network::Regtest => Some("bcrt1p"),
            _ => Some("tb1p"),
        },
        AddressFormat::Unknown => None,
    }
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    let unchecked = address
        .parse::<Address<NetworkUnchecked>>()
        .map_err(|error| LabError::InvalidAddress(format!("{error}")))?;
    let checked = unchecked
        .require_network(network)
        .map_err(|error| LabError::WrongNetwork(format!("{error}")))?;

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
    inspect_address(address, network).map(|report| report.script_pubkey_hex)
}
