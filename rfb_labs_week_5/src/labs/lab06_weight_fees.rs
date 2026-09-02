//! Lab 06 — calculate transaction weight, virtual size, and fees.

use crate::model::FeeComparison;
use crate::{LabError, LabResult};

/// BIP141 charges the non-witness bytes four times and the witness bytes once.
const WITNESS_SCALE_FACTOR: u64 = 4;

/// Calculate BIP141 weight from stripped and total serialized sizes.
///
/// `stripped_size` is the serialization without witness data, so it can never exceed
/// `total_size`. Weight is `stripped * 3 + total`, which is the same as
/// `stripped * 4 + witness_bytes`.
pub fn transaction_weight(stripped_size: u64, total_size: u64) -> LabResult<u64> {
    if stripped_size > total_size {
        return Err(LabError::InvalidSize(format!(
            "stripped size {stripped_size} exceeds total size {total_size}"
        )));
    }

    stripped_size
        .checked_mul(WITNESS_SCALE_FACTOR - 1)
        .and_then(|scaled| scaled.checked_add(total_size))
        .ok_or_else(|| LabError::InvalidSize(format!("weight overflows for total {total_size}")))
}

/// Calculate virtual size as `ceil(weight / 4)`.
pub fn virtual_size(weight: u64) -> u64 {
    // Round up so a transaction is never billed for less than the weight it uses.
    weight.div_ceil(WITNESS_SCALE_FACTOR)
}

/// Calculate a fee from virtual size and satoshis per virtual byte.
pub fn fee_sats(vbytes: u64, feerate_sat_vb: u64) -> LabResult<u64> {
    vbytes.checked_mul(feerate_sat_vb).ok_or_else(|| {
        LabError::InvalidSize(format!(
            "fee overflows for {vbytes} vB at {feerate_sat_vb} sat/vB"
        ))
    })
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
        // Saturating keeps the report honest if the SegWit template is the larger one.
        savings_sats: legacy_fee_sats.saturating_sub(segwit_fee_sats),
    })
}
