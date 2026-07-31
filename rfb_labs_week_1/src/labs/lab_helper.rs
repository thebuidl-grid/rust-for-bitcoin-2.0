use crate::{LabError, LabResult};
use serde_json::Value;

pub fn required_u32(value: &Value, field: &'static str) -> LabResult<u32> {
    let n = value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or(LabError::MissingField(field))?;

    u32::try_from(n).map_err(|_| LabError::MissingField(field))
}

pub fn required_bool(value: &Value, field: &'static str) -> LabResult<bool> {
    value
        .get(field)
        .and_then(Value::as_bool)
        .ok_or(LabError::MissingField(field))
}

pub fn required_array(value: &Value, field: &'static str) -> LabResult<Vec<Value>> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(ToOwned::to_owned)
        .ok_or(LabError::MissingField(field))
}
