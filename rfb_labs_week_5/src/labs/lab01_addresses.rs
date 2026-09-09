//! Lab 01 — identify Bitcoin address formats and enforce network safety.
use std::str::FromStr;

use bitcoin::{Network, Address, AddressType};
use bitcoin::address::NetworkUnchecked;

use crate::model::{AddressFormat, AddressReport};
use crate::{LabError, LabResult};

const PREFIX_TABLE: &[(&str, AddressFormat)] = &[
    ("bcrt1q", AddressFormat::P2wpkh),
    ("bcrt1p", AddressFormat::P2tr),
    ("tb1q", AddressFormat::P2wpkh),
    ("tb1p", AddressFormat::P2tr),
    ("bc1q", AddressFormat::P2wpkh),
    ("bc1p", AddressFormat::P2tr),
    ("1", AddressFormat::P2pkh),
    ("3", AddressFormat::P2sh),
    ("2", AddressFormat::P2sh),
    ("m", AddressFormat::P2pkh),
    ("n", AddressFormat::P2pkh),
];


/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
    // todo!("Lab 01: identify P2PKH, P2SH, P2WPKH, and P2TR prefixes")
    PREFIX_TABLE.iter().find(|(prefix, _)| address.starts_with(prefix)).map(|(_, format)| *format).unwrap_or(AddressFormat::Unknown)
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    // todo!("Lab 01: map address formats and networks to prefixes")
    match (format, network) {
        (AddressFormat::Unknown, _) => None,
        (AddressFormat::P2pkh, Network::Bitcoin) => Some("1"),
        (AddressFormat::P2pkh, Network::Testnet)|(AddressFormat::P2pkh, Network::Testnet4)|(AddressFormat::P2pkh, Network::Signet)|(AddressFormat::P2pkh, Network::Regtest) => Some("m/n"),
        (AddressFormat::P2sh, Network::Bitcoin) => Some("3"),
        (AddressFormat::P2sh, Network::Testnet)|(AddressFormat::P2sh, Network::Testnet4)|(AddressFormat::P2sh, Network::Signet)|(AddressFormat::P2sh, Network::Regtest) => Some("2"),
        (AddressFormat::P2wpkh, Network::Bitcoin) => Some("bc1q"),
        (AddressFormat::P2wpkh, Network::Testnet)|(AddressFormat::P2wpkh, Network::Testnet4)|(AddressFormat::P2wpkh, Network::Signet) => Some("tb1q"),
        (AddressFormat::P2tr, Network::Bitcoin) => Some("bc1p"),
        (AddressFormat::P2tr, Network::Testnet)|(AddressFormat::P2wpkh, Network::Testnet4)|(AddressFormat::P2wpkh, Network::Signet) => Some("tb1p"),
        (AddressFormat::P2wpkh, Network::Regtest) => Some("bcrt1q"),
        (AddressFormat::P2tr, Network::Regtest) => Some("bcrt1p"),
        (AddressFormat::P2tr, Network::Testnet4) | (AddressFormat::P2tr, Network::Signet) => todo!(),
    }
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    // todo!("Lab 01: validate the address and inspect its script type")
    let checked = parse_checked(address, network)?;

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
    // todo!("Lab 01: translate a checked address into scriptPubKey bytes")
    let checked = parse_checked(address, network)?;
    Ok(checked.script_pubkey().to_hex_string())
}

 fn parse_checked(address: &str, network: Network) -> LabResult<Address> {
     let unchecked: Address<NetworkUnchecked> =
         Address::from_str(address).map_err(|error| LabError::InvalidAddress(error.to_string()))?;

     unchecked
         .require_network(network)
         .map_err(|error| LabError::WrongNetwork(error.to_string()))
 }
