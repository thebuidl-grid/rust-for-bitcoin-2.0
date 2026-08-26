//! Lab 05 — reason about sender support across address generations.

use crate::model::{AddressFormat, CompatibilityReport, SenderCapabilities};

/// Return whether the sender can decode and pay the selected address format.
///
/// A sender needs the *encoding* that a format uses, not the ability to spend it:
/// P2SH addresses are Base58Check just like P2PKH, so any Base58-aware wallet can pay
/// a `3...` address whether or not it understands what is wrapped inside.
pub fn can_send_to(capabilities: SenderCapabilities, format: AddressFormat) -> bool {
    match format {
        AddressFormat::P2pkh => capabilities.base58_p2pkh,
        AddressFormat::P2sh => capabilities.base58_p2sh,
        AddressFormat::P2wpkh => capabilities.bech32,
        AddressFormat::P2tr => capabilities.bech32m,
        AddressFormat::Unknown => false,
    }
}

/// Summarize support for legacy, wrapped SegWit, native SegWit, and Taproot.
pub fn compatibility_report(capabilities: SenderCapabilities) -> CompatibilityReport {
    CompatibilityReport {
        p2pkh: can_send_to(capabilities, AddressFormat::P2pkh),
        p2sh_p2wpkh: can_send_to(capabilities, AddressFormat::P2sh),
        p2wpkh: can_send_to(capabilities, AddressFormat::P2wpkh),
        p2tr: can_send_to(capabilities, AddressFormat::P2tr),
    }
}

/// Choose the best supported single-key receiving format: prefer Taproot, then native
/// P2WPKH, then wrapped SegWit (P2SH-P2WPKH), then legacy P2PKH.
pub fn best_supported_format(capabilities: SenderCapabilities) -> Option<AddressFormat> {
    [
        AddressFormat::P2tr,
        AddressFormat::P2wpkh,
        AddressFormat::P2sh,
        AddressFormat::P2pkh,
    ]
    .into_iter()
    .find(|&format| can_send_to(capabilities, format))
}

/// Explain the encoding requirement that controls sender compatibility.
pub fn required_encoding(format: AddressFormat) -> &'static str {
    match format {
        AddressFormat::P2pkh | AddressFormat::P2sh => "Base58Check",
        AddressFormat::P2wpkh => "Bech32",
        AddressFormat::P2tr => "Bech32m",
        AddressFormat::Unknown => "unknown",
    }
}
