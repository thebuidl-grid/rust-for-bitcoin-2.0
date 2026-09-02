//! Lab 06 — calculate transaction weight, virtual size, and fees.

use crate::model::FeeComparison;
use crate::LabResult;

/// Calculate BIP141 weight from stripped and total serialized sizes.
pub fn transaction_weight(stripped_size: u64, total_size: u64) -> LabResult<u64> {
    if stripped_size > total_size {
        return Err(crate::LabError::InvalidSize(
            "stripped size cannot exceed total size".to_owned(),
        ));
    }

    let stripped_weight = stripped_size
        .checked_mul(3)
        .ok_or_else(|| crate::LabError::InvalidSize("weight overflow".to_owned()))?;

    stripped_weight
        .checked_add(total_size)
        .ok_or_else(|| crate::LabError::InvalidSize("weight overflow".to_owned()))
}

/// Calculate virtual size as `ceil(weight / 4)`.
pub fn virtual_size(weight: u64) -> u64 {
    weight / 4 + u64::from(!weight.is_multiple_of(4))
}
/// Calculate a fee from virtual size and satoshis per virtual byte.
pub fn fee_sats(vbytes: u64, feerate_sat_vb: u64) -> LabResult<u64> {
    vbytes
        .checked_mul(feerate_sat_vb)
        .ok_or_else(|| crate::LabError::InvalidSize("fee overflow".to_owned()))
}

/// Compare illustrative legacy and native-SegWit transactions at one feerate.
pub fn compare_fees(
    legacy_vbytes: u64,
    segwit_vbytes: u64,
    feerate_sat_vb: u64,
) -> LabResult<FeeComparison> {
    let legacy_fee_sats = fee_sats(legacy_vbytes, feerate_sat_vb)?;
    let segwit_fee_sats = fee_sats(segwit_vbytes, feerate_sat_vb)?;

    let savings_sats = legacy_fee_sats
        .checked_sub(segwit_fee_sats)
        .ok_or_else(|| {
            crate::LabError::InvalidSize(
                "SegWit fee cannot exceed legacy fee in this comparison".to_owned(),
            )
        })?;

    Ok(FeeComparison {
        legacy_vbytes,
        segwit_vbytes,
        legacy_fee_sats,
        segwit_fee_sats,
        savings_sats,
    })
}
