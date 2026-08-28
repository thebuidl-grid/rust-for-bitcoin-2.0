//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use std::ptr::addr_eq;

use bitcoin::Network;

use crate::model::{AddressFormat, AddressReport};
use crate::LabResult;

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
 
// The methods below identifies mainnet, regtest, and testnets their suffixes are identified differently

//  1 / m / n       → P2PKH
// 3 / 2           → P2SH
// bc1q / tb1q     → P2WPKH
// bcrt1q          → P2WPKH
// bc1p / tb1p     → P2TR
// bcrt1p          → P2TR


let format = match address {
   value if  address.starts_with("1") || address.starts_with("m") || address.starts_with("n") => AddressFormat::P2pkh,
   value if address.starts_with("3") || address.starts_with("2") => AddressFormat::P2sh,
   value if address.starts_with("bc1q") || address.starts_with("tb1q") || address.starts_with("bcrt1q")=> AddressFormat::P2wpkh,
   value if address.starts_with("bc1p") || address.starts_with("tb1p") || address.starts_with("bcrt1p") => AddressFormat::P2tr,
   _ => AddressFormat::Unknown
};

format

}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    todo!("Lab 01: map address formats and networks to prefixes")
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    todo!("Lab 01: validate the address and inspect its script type")
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    todo!("Lab 01: translate a checked address into scriptPubKey bytes")
}
