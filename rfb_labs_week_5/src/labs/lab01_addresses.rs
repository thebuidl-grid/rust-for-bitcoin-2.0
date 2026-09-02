//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use crate::model::{AddressFormat, AddressReport};
use crate::{LabError, LabResult};
use bitcoin::Network;
use std::str::FromStr;

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    if address.starts_with("1") || address.starts_with("m") || address.starts_with("n") {
        return AddressFormat::P2pkh;
    } else if address.starts_with("2") || address.starts_with("3") {
        return AddressFormat::P2sh;
    } else if address.starts_with("bc1q")
        || address.starts_with("tb1q")
        || address.starts_with("bcrt1q")
    {
        return AddressFormat::P2wpkh;
    } else if address.starts_with("bc1p")
        || address.starts_with("tb1p")
        || address.starts_with("bcrt1p")
    {
        return AddressFormat::P2tr;
    }
    AddressFormat::Unknown
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    match (format, network) {
        //Mainnet
        (AddressFormat::P2pkh, Network::Bitcoin) => Some("1"),
        (AddressFormat::P2sh, Network::Bitcoin) => Some("3"),
        (AddressFormat::P2wpkh, Network::Bitcoin) => Some("bc1q"),
        (AddressFormat::P2tr, Network::Bitcoin) => Some("bc1p"),

        // Testnet
        (AddressFormat::P2pkh, Network::Testnet) => Some("m/n"),
        (AddressFormat::P2sh, Network::Testnet) => Some("2"),
        (AddressFormat::P2wpkh, Network::Testnet) => Some("tb1q"),
        (AddressFormat::P2tr, Network::Testnet) => Some("tb1p"),

        // Testnet4
        (AddressFormat::P2pkh, Network::Testnet4) => Some("t4"),
        (AddressFormat::P2sh, Network::Testnet4) => Some("2"),
        (AddressFormat::P2wpkh, Network::Testnet4) => Some("tb1q"),
        (AddressFormat::P2tr, Network::Testnet4) => Some("tb1p"),

        // Signet
        (AddressFormat::P2pkh, Network::Signet) => Some("m/n"),
        (AddressFormat::P2sh, Network::Signet) => Some("2"),
        (AddressFormat::P2wpkh, Network::Signet) => Some("tb1q"),
        (AddressFormat::P2tr, Network::Signet) => Some("tb1p"),

        //Regtest
        (AddressFormat::P2pkh, Network::Regtest) => Some("m/n"),
        (AddressFormat::P2sh, Network::Regtest) => Some("2"),
        (AddressFormat::P2wpkh, Network::Regtest) => Some("bcrt1q"),
        (AddressFormat::P2tr, Network::Regtest) => Some("bcrt1p"),

        //Unkown
        (AddressFormat::Unknown, _) => None,
    }
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    let unchecked_addr =
        bitcoin::Address::from_str(address).map_err(|e| LabError::InvalidAddress(e.to_string()))?;
    let addr = unchecked_addr
        .require_network(network)
        .map_err(|e| LabError::WrongNetwork(e.to_string()))?;

    let script_pubkey = addr.script_pubkey();

    let format = identify_prefix(address);
    Ok(AddressReport {
        address: address.to_string(),
        network: network.to_string(),
        format,
        script_pubkey_hex: hex::encode(script_pubkey.as_bytes()),
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    let report = inspect_address(address, network)?;
    Ok(report.script_pubkey_hex)
}
