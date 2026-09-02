//! Lab 01 — identify Bitcoin address formats and enforce network safety.

use std::str::FromStr;

use bitcoin::address::{Address, AddressType, NetworkUnchecked};
use bitcoin::Network;

use crate::model::{AddressFormat, AddressReport};
use crate::{LabError, LabResult};

/// Human-readable parts for the networks these labs touch, longest first so that
/// `bcrt` is matched before the `bc` it starts with.
const KNOWN_HRPS: [&str; 3] = ["bcrt", "bc", "tb"];

/// Characters that follow the `1` separator once the witness version is encoded.
const WITNESS_V0: char = 'q';
const WITNESS_V1: char = 'p';

/// Number of characters a bech32 address adds after its human-readable part when the
/// witness program is 20 bytes: the `1` separator, the version character, 32 data
/// characters, and the 6-character checksum.
const P2WPKH_TAIL: usize = 40;

/// Identify an address family from its human-readable prefix.
///
/// This is deliberately a prefix-only guess. It never checks a checksum, so it will
/// classify a corrupted or fabricated string too. [`inspect_address`] does the real
/// validation.
pub fn identify_prefix(address: &str) -> AddressFormat {
    let candidate = address.trim().to_lowercase();

    for hrp in KNOWN_HRPS {
        let separator = format!("{hrp}1");
        let Some(data) = candidate.strip_prefix(&separator) else {
            continue;
        };

        return match data.chars().next() {
            // Witness version 1 is Taproot, and P2TR is its only defined output type.
            Some(WITNESS_V1) => AddressFormat::P2tr,
            // Witness version 0 covers P2WPKH and P2WSH. Only the length separates
            // them, so anything longer than a 20-byte program is not single-key.
            Some(WITNESS_V0) if candidate.len() == hrp.len() + P2WPKH_TAIL => AddressFormat::P2wpkh,
            _ => AddressFormat::Unknown,
        };
    }

    match address.trim().chars().next() {
        // Base58Check version bytes 0x00 (mainnet) and 0x6f (test networks).
        Some('1') | Some('m') | Some('n') => AddressFormat::P2pkh,
        // Base58Check version bytes 0x05 (mainnet) and 0xc4 (test networks).
        Some('3') | Some('2') => AddressFormat::P2sh,
        _ => AddressFormat::Unknown,
    }
}

/// Return the expected human-readable prefix for a format on a selected network.
pub fn expected_prefix(format: AddressFormat, network: Network) -> Option<&'static str> {
    let prefix = match (format, network) {
        (AddressFormat::P2pkh, Network::Bitcoin) => "1",
        (AddressFormat::P2pkh, _) => "m/n",
        (AddressFormat::P2sh, Network::Bitcoin) => "3",
        (AddressFormat::P2sh, _) => "2",
        (AddressFormat::P2wpkh, Network::Bitcoin) => "bc1q",
        (AddressFormat::P2wpkh, Network::Regtest) => "bcrt1q",
        (AddressFormat::P2wpkh, _) => "tb1q",
        (AddressFormat::P2tr, Network::Bitcoin) => "bc1p",
        (AddressFormat::P2tr, Network::Regtest) => "bcrt1p",
        (AddressFormat::P2tr, _) => "tb1p",
        (AddressFormat::Unknown, _) => return None,
    };

    Some(prefix)
}

/// Parse an address, reject the wrong network, and return its full report.
pub fn inspect_address(address: &str, network: Network) -> LabResult<AddressReport> {
    let checked = parse_checked(address, network)?;

    Ok(AddressReport {
        address: checked.to_string(),
        network: network.to_string(),
        format: classify(&checked),
        script_pubkey_hex: checked.script_pubkey().to_hex_string(),
    })
}

/// Return the scriptPubKey encoded by a network-checked address.
pub fn script_pubkey_hex(address: &str, network: Network) -> LabResult<String> {
    Ok(parse_checked(address, network)?
        .script_pubkey()
        .to_hex_string())
}

/// Decode the Base58Check or bech32 payload, then require the intended network.
///
/// The two failures are reported separately: a bad checksum is a malformed address,
/// while a good checksum on the wrong network is a live-funds hazard.
fn parse_checked(address: &str, network: Network) -> LabResult<Address> {
    Address::<NetworkUnchecked>::from_str(address.trim())
        .map_err(|error| LabError::InvalidAddress(error.to_string()))?
        .require_network(network)
        .map_err(|error| LabError::WrongNetwork(error.to_string()))
}

/// Read the script family from the decoded payload rather than from the text.
fn classify(address: &Address) -> AddressFormat {
    match address.address_type() {
        Some(AddressType::P2pkh) => AddressFormat::P2pkh,
        Some(AddressType::P2sh) => AddressFormat::P2sh,
        Some(AddressType::P2wpkh) => AddressFormat::P2wpkh,
        Some(AddressType::P2tr) => AddressFormat::P2tr,
        _ => AddressFormat::Unknown,
    }
}
