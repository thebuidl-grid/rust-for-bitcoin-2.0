//! Lab 06 — calculate transaction weight, virtual size, and fees.

use crate::model::FeeComparison;
use crate::LabResult;
use crate::LabError;

/// Calculate BIP141 weight from stripped and total serialized sizes.
/// weight = stripped_size * 3 + total_size
/// stripped_size must be <= total_size (it excludes witness data).
pub fn transaction_weight(stripped_size: u64, total_size: u64) -> LabResult<u64> {
    if stripped_size > total_size {
        return Err(LabError::InvalidSize(
            "stripped_size cannot be greater than total_size".to_string(),
        ));
    }
    Ok(stripped_size * 3 + total_size)
}

/// Calculate virtual size as `ceil(weight / 4)`.
pub fn virtual_size(weight: u64) -> u64 {
    (weight + 3) / 4
}

/// Calculate a fee from virtual size and satoshis per virtual byte.
pub fn fee_sats(vbytes: u64, feerate_sat_vb: u64) -> LabResult<u64> {
    vbytes
        .checked_mul(feerate_sat_vb)
        .ok_or_else(|| LabError::InvalidSize("fee calculation overflowed u64".to_string()))
}

/// Compare illustrative legacy and native-SegWit transactions at one feerate.
pub fn compare_fees(
    legacy_vbytes: u64,
    segwit_vbytes: u64,
    feerate_sat_vb: u64,
) -> LabResult<FeeComparison> {
    let legacy_fee_sats = fee_sats(legacy_vbytes, feerate_sat_vb)?;
    let segwit_fee_sats = fee_sats(segwit_vbytes, feerate_sat_vb)?;
    let savings_sats = legacy_fee_sats.saturating_sub(segwit_fee_sats);

    Ok(FeeComparison {
        legacy_vbytes,
        segwit_vbytes,
        legacy_fee_sats,
        segwit_fee_sats,
        savings_sats,
    })
}
