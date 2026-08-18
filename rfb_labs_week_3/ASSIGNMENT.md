# Week 3 Assignment — Contribute to rust-bitcoin

The major goal of this program is to contribute meaningfully to Bitcoin open-source
projects. This week you will select an issue from the
[rust-bitcoin](https://github.com/rust-bitcoin/rust-bitcoin) repository, implement
a fix, and open a pull request.

## Assigned issue

**Issue:** Any type that has a `FromStr` should have `TryFrom<{&str, String, Box<str>,
Rc<str>, Arc<str}>`.

**Affected types:**
- `bitcoin-units::Amount`
- `bitcoin-units::SignedAmount`

**Scope:** Implement the missing `TryFrom` conversions in the `units` crate of a
rust-bitcoin fork. The implementation should delegate to the existing `FromStr`
parsing. You must not change public type names or function signatures in ways that
break the existing API.

## Required work

- [ ] **Part 1 — Fork and clone:** Fork `rust-bitcoin/rust-bitcoin` on GitHub, clone
  your fork locally, and create a feature branch named `tryfrom-amount-string`.
- [ ] **Part 2 — Implement `TryFrom<&str>`:** Add `TryFrom<&str>` for `Amount` and
  `SignedAmount`. Both types already implement `FromStr` and return `ParseError`,
  so the implementation is a one-line delegation.
- [ ] **Part 3 — Implement `TryFrom<String>`:** Add `TryFrom<String>` for both types.
  Delegate to `Self::from_str(&s)` or `Self::from_str(s.as_str())`. Gate the
  implementation behind `#[cfg(feature = "alloc")]` because `String` requires the
  `alloc` feature.
- [ ] **Part 4 — Implement `TryFrom<Box<str>>`:** Add `TryFrom<Box<str>>` for both
  types. Also gate behind `#[cfg(feature = "alloc")]`.
- [ ] **Part 5 — Tests:** Add unit tests in the `units` crate verifying that each
  conversion works for valid Bitcoin-denominated strings (e.g. `"1.5 BTC"`,
  `"50000 sats"`, `"0.01 BTC"`) and that invalid strings return the correct
  `ParseError`.
- [ ] **Part 6 — Clippy and fmt:** Run `cargo fmt --check` and
  `cargo clippy --all-targets --all-features -- -D warnings` in the `units` crate.
  Fix all warnings before submitting.
- [ ] **Part 7 — Pull request:** Push your feature branch and open a PR against
  `rust-bitcoin/rust-bitcoin:main`. The PR description should link the assigned
  issue, summarise the change, and include the test output.
- [ ] **Part 8 — Evidence:** Record the commands run, the diff, test output, and the
  PR URL in `WORK_DONE.md`.

## Testing checklist

- Valid string conversions parse correctly for both `Amount` and `SignedAmount`.
- Invalid strings (e.g. `"not-a-number"`, `"1.2.3 BTC"`) return `Err(ParseError)`.
- `cargo test -p bitcoin-units` passes in the forked repository.
- `cargo fmt --check` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.

## Submission standard

- The PR is open against `rust-bitcoin/rust-bitcoin:main`.
- All required `TryFrom` implementations are present and tested.
- `cargo fmt --check` passes in the `units` crate.
- `cargo clippy --all-targets --all-features -- -D warnings` passes in the `units` crate.
- `WORK_DONE.md` in this folder contains commands run, diffs, test output, and the PR URL.
- You may not add an external Bitcoin library; the goal is to practise reading
  real open-source code, writing idiomatic Rust, and going through code review.

## Design notes

`Amount` and `SignedAmount` live in `bitcoin-units` under the `units` crate. They
already implement `core::str::FromStr` and return `ParseError`. Adding `TryFrom`
is a pure API-surface improvement: it lets callers write `amount.try_from("1 BTC")?`
instead of `"1 BTC".parse::<Amount>()?`.

Because `String`, `Box<str>`, and friends require `alloc`, the implementations must
be gated with `#[cfg(feature = "alloc")]` to preserve `no-std` compatibility. The
`&str` implementation is always available.

## Example output

```rust
use bitcoin_units::Amount;

let amount = Amount::try_from("1.5 BTC")?;
assert_eq!(amount.to_sat(), 150_000_000);

let amount = Amount::try_from("50000 sats".to_string())?;
assert_eq!(amount.to_sat(), 50_000);

let amount = Amount::try_from("0.01 BTC".to_string())?;
assert_eq!(amount.to_sat(), 1_000_000);
```
