//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use bitcoin::address::NetworkUnchecked;
use bitcoin::{Address, AddressType, Network};

use crate::model::{AddressFormat, AddressReport};
use crate::{LabError, LabResult};

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    // The methods below identifies mainnet, regtest, and testnets their suffixes are identified differently

    //  1 / m / n      → P2PKH
    // 3 / 2           → P2SH
    // bc1q / tb1q     → P2WPKH
    // bcrt1q          → P2WPKH
    // bc1p / tb1p     → P2TR
    // bcrt1p          → P2TR

    match address {
        value if value.starts_with("1") || value.starts_with("m") || value.starts_with("n") => {
            AddressFormat::P2pkh
        }
        value if value.starts_with("3") || value.starts_with("2") => AddressFormat::P2sh,
        value
            if value.starts_with("bc1q")
                || value.starts_with("tb1q")
                || value.starts_with("bcrt1q") =>
        {
            AddressFormat::P2wpkh
        }
        value
            if value.starts_with("bc1p")
                || value.starts_with("tb1p")
                || value.starts_with("bcrt1p") =>
        {
            AddressFormat::P2tr
        }
        _ => AddressFormat::Unknown,
    }
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    match (format, network) {
        (
            AddressFormat::P2pkh,
            Network::Regtest | Network::Signet | Network::Testnet | Network::Testnet4,
        ) => Some("m/n"),
        (
            AddressFormat::P2sh,
            Network::Regtest | Network::Signet | Network::Testnet | Network::Testnet4,
        ) => Some("2"),
        (AddressFormat::P2wpkh, Network::Regtest) => Some("bcrt1q"),
        (AddressFormat::P2tr, Network::Regtest) => Some("bcrt1p"),
        (AddressFormat::P2wpkh, Network::Testnet | Network::Signet | Network::Testnet4) => {
            Some("tb1q")
        }
        (AddressFormat::P2tr, Network::Testnet | Network::Signet | Network::Testnet4) => {
            Some("tb1p")
        }
        (AddressFormat::P2pkh, Network::Bitcoin) => Some("1"),
        (AddressFormat::P2sh, Network::Bitcoin) => Some("3"),
        (AddressFormat::P2wpkh, Network::Bitcoin) => Some("bc1q"),
        (AddressFormat::P2tr, Network::Bitcoin) => Some("bc1p"),
        (AddressFormat::Unknown, _) => None,
    }
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    //  Parse address
    let unchecked = address
        .parse::<Address<NetworkUnchecked>>()
        .map_err(|err| LabError::InvalidAddress(err.to_string()))?;

    // Check network
    let checked_address = unchecked
        .require_network(network)
        .map_err(|err| LabError::WrongNetwork(err.to_string()))?;

    // Identify validated address type

    let address_type = checked_address.address_type();

    let format = match address_type {
        Some(AddressType::P2pkh) => AddressFormat::P2pkh,
        Some(AddressType::P2sh) => AddressFormat::P2sh,
        Some(AddressType::P2wpkh) => AddressFormat::P2wpkh,
        Some(AddressType::P2tr) => AddressFormat::P2tr,
        _ => AddressFormat::Unknown,
    };

    // 4. Build AddressReport
    Ok(AddressReport {
        address: checked_address.to_string(),
        format,
        network: network.to_string(),
        script_pubkey_hex: checked_address.script_pubkey().to_hex_string(),
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    let report = inspect_address(address, network)?;

    Ok(report.address)
}
