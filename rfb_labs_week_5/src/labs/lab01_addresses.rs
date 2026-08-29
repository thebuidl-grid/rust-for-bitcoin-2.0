//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use std::str::FromStr;

use bitcoin::{Address, AddressType, Network};

use crate::model::{AddressFormat, AddressReport};
use crate::{LabError, LabResult};

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    let address = address.trim();

    match address.chars().next() {
        Some('1') => return AddressFormat::P2pkh,
        Some('2') | Some('3') => return AddressFormat::P2sh,
        _ => {}
    }

    // Bech32/Bech32m addresses are `<hrp>1<data>`, and the first data character encodes
    // the witness version (`q` = 0, `p` = 1). Match on the known HRPs used in these labs
    // rather than the last '1' in the string: '1' is itself a valid Base58 character, so
    // a legacy m/n/1/2/3 address can contain one by coincidence and must not be confused
    // for a bech32 separator.
    let bech32_data = address
        .strip_prefix("bc1")
        .or_else(|| address.strip_prefix("tb1"))
        .or_else(|| address.strip_prefix("bcrt1"));

    if let Some(data) = bech32_data {
        match data.chars().next() {
            Some('q') => return AddressFormat::P2wpkh,
            Some('p') => return AddressFormat::P2tr,
            _ => {}
        }
    }

    AddressFormat::Unknown
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

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    let parsed = Address::from_str(address)
        .map_err(|error| LabError::InvalidAddress(error.to_string()))?
        .require_network(network)
        .map_err(|error| LabError::WrongNetwork(error.to_string()))?;

    let format = match parsed.address_type() {
        Some(AddressType::P2pkh) => AddressFormat::P2pkh,
        Some(AddressType::P2sh) => AddressFormat::P2sh,
        Some(AddressType::P2wpkh) => AddressFormat::P2wpkh,
        Some(AddressType::P2tr) => AddressFormat::P2tr,
        _ => AddressFormat::Unknown,
    };

    Ok(AddressReport {
        address: parsed.to_string(),
        network: network.to_string(),
        format,
        script_pubkey_hex: parsed.script_pubkey().to_hex_string(),
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    let parsed = Address::from_str(address)
        .map_err(|error| LabError::InvalidAddress(error.to_string()))?
        .require_network(network)
        .map_err(|error| LabError::WrongNetwork(error.to_string()))?;

    Ok(parsed.script_pubkey().to_hex_string())
}
