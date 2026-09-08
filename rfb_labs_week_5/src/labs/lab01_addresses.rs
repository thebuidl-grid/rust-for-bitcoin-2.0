//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use bitcoin::{Network, network};
use std::str::FromStr;

use crate::model::{AddressFormat, AddressReport};
use crate::LabResult;
use crate::LabError;

/// Identify an address family from its human-readable prefix.
pub fn identify_prefix(address: &str) -> AddressFormat {
   
    if address.starts_with('1')
       || address.starts_with('m')
       || address.starts_with('n') {
        AddressFormat::P2pkh
    } else if address.starts_with('3') {
        AddressFormat::P2sh
    } else if address.starts_with("bc1q")
        || address.starts_with("bcrt1q")
        || address.starts_with("tb1q")
    {
        AddressFormat::P2wpkh
    } else if address.starts_with("bc1p")
        || address.starts_with("bcrt1p")
        || address.starts_with("tb1p")
    {
        AddressFormat::P2tr
    } else {
        AddressFormat::Unknown
    }
    
    // todo!("Lab 01: identify P2PKH, P2SH, P2WPKH, and P2TR prefixes")
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {

    match (format, network) {
        (AddressFormat::P2pkh, Network::Bitcoin) => Some("1"),
        (AddressFormat::P2pkh, Network::Testnet) => Some("m/n"),
        (AddressFormat::P2pkh, Network::Regtest) => Some("m/n"),

        (AddressFormat::P2sh, Network::Bitcoin) => Some("3"),
        (AddressFormat::P2sh, Network::Testnet) => Some("2"),
        (AddressFormat::P2sh, Network::Regtest) => Some("2"),
        
        (AddressFormat::P2wpkh, Network::Bitcoin) => Some("bc1q"),
        (AddressFormat::P2wpkh, Network::Testnet) => Some("tb1q"),
        (AddressFormat::P2wpkh, Network::Regtest) => Some("bcrt1q"),
        
        (AddressFormat::P2tr, Network::Bitcoin) => Some("bc1p"),
        (AddressFormat::P2tr, Network::Testnet) => Some("tb1p"),
        (AddressFormat::P2tr, Network::Regtest) => Some("bcrt1p"),
        _ => None
    }    

    // todo!("Lab 01: map address formats and networks to prefixes")
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {

    let address_with_unchecked_network = bitcoin::Address::from_str(address)
    .map_err(|e| LabError::InvalidAddress(e.to_string()))?;

    let address_with_checked_network = address_with_unchecked_network.require_network(network)
    .map_err(|e| LabError::WrongNetwork(e.to_string()))?;

    let format = identify_prefix(address);
    let script_pubkey_hex = address_with_checked_network.script_pubkey().to_hex_string();
    let network_str = format!("{}", network);

    Ok(AddressReport { 
        address: address.to_string(), 
        network: network_str, 
        format, 
        script_pubkey_hex 
    })

        // todo!("Lab 01: validate the address and inspect its script type")
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {

    let address_with_unchecked_network = bitcoin::Address::from_str(address)
    .map_err(|e| LabError::InvalidAddress(e.to_string()))?;

    let address_with_checked_network = address_with_unchecked_network
    .require_network(network).map_err(|e| LabError::WrongNetwork(e.to_string()))?;

    let script_pubkey_hex = address_with_checked_network.script_pubkey().to_hex_string();

    Ok(script_pubkey_hex)

    //////////////////// OR

    // let address = inspect_address(address, network)?;
    // Ok(address.script_pubkey_hex)

}
