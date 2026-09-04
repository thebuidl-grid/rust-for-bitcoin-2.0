//! Lab 06 — calculate transaction weight, virtual size, and fees.

use crate::model::FeeComparison;
use crate::LabResult;

/// Calculate BIP141 weight from stripped and total serialized sizes.
pub fn transaction_weight(stripped_size: u64, total_size: u64) -> LabResult<u64> {
    if total_size < stripped_size {
        return Err(crate::error::LabError::InvalidSize(
            "total size cannot be smaller than stripped size".to_owned(),
        ));
    }
    stripped_size
        .checked_mul(3)
        .and_then(|weight| weight.checked_add(total_size))
        .ok_or_else(|| crate::error::LabError::InvalidSize("weight overflow".to_owned()))
}

/// Calculate virtual size as `ceil(weight / 4)`.
pub fn virtual_size(weight: u64) -> u64 {
    weight.saturating_add(3) / 4
}

/// Calculate a fee from virtual size and satoshis per virtual byte.
pub fn fee_sats(vbytes: u64, feerate_sat_vb: u64) -> LabResult<u64> {
    vbytes
        .checked_mul(feerate_sat_vb)
        .ok_or_else(|| crate::error::LabError::InvalidSize("fee overflow".to_owned()))
}

/// Compare illustrative legacy and native-SegWit transactions at one feerate.
pub fn compare_fees(
    legacy_vbytes: u64,
    segwit_vbytes: u64,
    feerate_sat_vb: u64,
) -> LabResult<FeeComparison> {
    let legacy_fee_sats = fee_sats(legacy_vbytes, feerate_sat_vb)?;
    let segwit_fee_sats = fee_sats(segwit_vbytes, feerate_sat_vb)?;
    Ok(FeeComparison {
        legacy_vbytes,
        segwit_vbytes,
        legacy_fee_sats,
        segwit_fee_sats,
        savings_sats: legacy_fee_sats.saturating_sub(segwit_fee_sats),
    })
}
