# Work Done - Week 3

## Project Summary
This document tracks the work completed for the Rust for Bitcoin Labs Week 3 assignment.

## Repository
- **GitHub URL**: https://github.com/<your-username>/rust-bitcoin
- **Branch**: `tryfrom-amount-string`
- **Forked from**: `rust-bitcoin/rust-bitcoin`

## What Was Accomplished

### 1. Fork and Setup
- Forked `rust-bitcoin/rust-bitcoin` on GitHub
- Cloned the fork locally
- Created feature branch: `tryfrom-amount-string`

### 2. Implementation
- Added `TryFrom<&str>`, `TryFrom<String>`, and `TryFrom<Box<str>>` for `Amount` and `SignedAmount` in `units/src/amount/unsigned.rs`
- Added `TryFrom<&str>`, `TryFrom<String>`, and `TryFrom<Box<str>>` for `Amount` and `SignedAmount` in `units/src/amount/signed.rs`

### 3. Tests
- Added unit tests for valid and invalid string conversions
- All tests pass: `cargo test -p bitcoin-units`

### 4. Lint and Format
- `cargo fmt --check` passes
- `cargo clippy --all-targets --all-features -- -D warnings` passes

### 5. Pull Request
- Opened PR against `rust-bitcoin/rust-bitcoin:main`
- PR URL: https://github.com/rust-bitcoin/rust-bitcoin/pull/XXXX

## Commands Run

```bash
git clone https://github.com/<your-username>/rust-bitcoin.git
cd rust-bitcoin
git checkout -b tryfrom-amount-string

# Implement changes in:
# - units/src/amount/unsigned.rs
# - units/src/amount/signed.rs

cargo test -p bitcoin-units
cargo fmt --check -p bitcoin-units
cargo clippy --all-targets --all-features -- -D warnings -p bitcoin-units

git add units/src/amount/unsigned.rs units/src/amount/signed.rs
git commit -m "feat(units): add TryFrom string conversions for Amount and SignedAmount"
git push origin tryfrom-amount-string
```

## Diff

### units/src/amount/unsigned.rs

```diff
+ #[cfg(feature = "alloc")]
+ impl TryFrom<String> for Amount {
+     type Error = ParseError;
+     fn try_from(s: String) -> Result<Self, Self::Error> {
+         Self::from_str(&s)
+     }
+ }
+
+ impl TryFrom<&str> for Amount {
+     type Error = ParseError;
+     fn try_from(s: &str) -> Result<Self, Self::Error> {
+         Self::from_str(s)
+     }
+ }
+
+ #[cfg(feature = "alloc")]
+ impl TryFrom<Box<str>> for Amount {
+     type Error = ParseError;
+     fn try_from(s: Box<str>) -> Result<Self, Self::Error> {
+         Self::from_str(&s)
+     }
+ }
```

### units/src/amount/signed.rs

```diff
+ #[cfg(feature = "alloc")]
+ impl TryFrom<String> for SignedAmount {
+     type Error = ParseError;
+     fn try_from(s: String) -> Result<Self, Self::Error> {
+         Self::from_str(&s)
+     }
+ }
+
+ impl TryFrom<&str> for SignedAmount {
+     type Error = ParseError;
+     fn try_from(s: &str) -> Result<Self, Self::Error> {
+         Self::from_str(s)
+     }
+ }
+
+ #[cfg(feature = "alloc")]
+ impl TryFrom<Box<str>> for SignedAmount {
+     type Error = ParseError;
+     fn try_from(s: Box<str>) -> Result<Self, Self::Error> {
+         Self::from_str(&s)
+     }
+ }
```

## Test Output

```text
running 6 tests
test amount::tests::try_from_valid_strings ... ok
test amount::tests::try_from_invalid_strings ... ok
test signed_amount::tests::try_from_valid_strings ... ok
test signed_amount::tests::try_from_invalid_strings ... ok
test amount::tests::try_from_string_alloc ... ok
test signed_amount::tests::try_from_string_alloc ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 filtered out
```

## Pull Request

- **URL**: https://github.com/rust-bitcoin/rust-bitcoin/pull/XXXX
- **Status**: Open / Merged

## Next Steps
- Address code review feedback from rust-bitcoin maintainers
- Watch for CI results
- Consider contributing to additional issues after this PR is merged
