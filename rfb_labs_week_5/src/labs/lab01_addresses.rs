//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use bitcoin::{Address, Network};
use std::str::FromStr;

use crate::model::{AddressFormat, AddressReport};
use crate::{LabError, LabResult};

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    // todo!("Lab 01: identify P2PKH, P2SH, P2WPKH, and P2TR prefixes")
    if address.starts_with("bc1q") {
        return AddressFormat::P2wpkh;
    } else if address.starts_with("bc1p") {
        return AddressFormat::P2tr;
    }

    match &address[0..1] {
        "1" | "m" => AddressFormat::P2pkh,
        "3" | "n" => AddressFormat::P2sh,
        _ => AddressFormat::Unknown,
    }
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    // todo!("Lab 01: map address formats and networks to prefixes")
    match (format, network) {
        (AddressFormat::P2pkh, Network::Regtest) => Some("m/n"),
        (AddressFormat::P2sh, Network::Regtest) => Some("2"),
        (AddressFormat::P2wpkh, Network::Regtest) => Some("bcrt1q"),
        (AddressFormat::P2tr, Network::Regtest) => Some("bcrt1p"),
        _ => Some("unknown"),
    }
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    // todo!("Lab 01: validate the address and inspect its script type")
    let parsed =
        Address::from_str(address).map_err(|_| LabError::InvalidAddress(address.to_string()))?;

    let checked = parsed
        .require_network(network)
        .map_err(|e| LabError::WrongNetwork(e.to_string()))?;

    let format = identify_prefix(address);

    Ok(AddressReport {
        address: address.to_string(),
        network: format!("{network:?}").to_lowercase(),
        format,
        script_pubkey_hex: checked.script_pubkey().to_hex_string(),
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    // todo!("Lab 01: translate a checked address into scriptPubKey bytes")
    let address = inspect_address(address, network)?;
    Ok(address.script_pubkey_hex)
}
