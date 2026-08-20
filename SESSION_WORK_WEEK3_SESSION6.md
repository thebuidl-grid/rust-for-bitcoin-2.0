# Session Work — Week 3 Session 6: Assignment Setup

**Date:** 2026-08-18  
**Session:** Rust for Bitcoin Week 3 — Session 6  
**Topic:** Assignment scaffolding for rust-bitcoin contribution

## What I did

### 1. Analysed the repository

- Inspected `rust-for-bitcoin-2.0` layout: `rfb_labs_week_1` (labs), `rfb_labs_week_2` (transaction assignment), top-level `README.md`.
- Read `rfb_labs_week_2/ASSIGNMENT.md`, `README.md`, `WORK_DONE.md`, `src/lib.rs`, and `src/transaction.rs`.
- Reviewed git history and branches (`rust-for-bitcoin-3.0`, `rust-for-bitcoin-3.00`, `main`).
- Confirmed Week 2 is complete and the repository is ready for Week 3 work.

### 2. Identified the Week 3 assignment

- Based on the session email text and the Week 2/Week 3 progression pattern, determined the Week 3 assignment is a rust-bitcoin open-source contribution.
- Selected a concrete, realistic issue: **“Any type that has a `FromStr` should have `TryFrom<{&str, String, Box<str>, Rc<str>, Arc<str}>`”** for `bitcoin-units::Amount` and `SignedAmount`.
- Verified the issue scope against the rust-bitcoin `units` crate structure and feature flags (`alloc`, `std`, `no-std`).

### 3. Created `rfb_labs_week_3`

Created the Week 3 folder and all required files:

| File | Purpose |
|------|---------|
| `rfb_labs_week_3/Cargo.toml` | Package manifest for the Week 3 scaffold |
| `rfb_labs_week_3/src/lib.rs` | Starter library documenting the assignment context |
| `rfb_labs_week_3/ASSIGNMENT.md` | Full assignment spec: parts 1–8, testing checklist, submission standard, design notes, example output |
| `rfb_labs_week_3/README.md` | Recommended workflow, commands, PR checklist |
| `rfb_labs_week_3/WORK_DONE.md` | Evidence template: commands, diff, test output, PR URL placeholders |
| `rfb_labs_week_3/SUBMISSION.md` | Session 6 email notification with deadline and submission instructions |

### 4. Validated the scaffold

- Ran `cargo fmt --check` in `rfb_labs_week_3` — passes.
- Ran `cargo check` in `rfb_labs_week_3` — passes.

## Files changed

```
rfb_labs_week_3/Cargo.toml      (new)
rfb_labs_week_3/src/lib.rs      (new)
rfb_labs_week_3/ASSIGNMENT.md   (new)
rfb_labs_week_3/README.md       (new)
rfb_labs_week_3/WORK_DONE.md    (new)
rfb_labs_week_3/SUBMISSION.md   (new)
```

## Next steps for the student

1. Fork `rust-bitcoin/rust-bitcoin` on GitHub.
2. Clone the fork and create branch `tryfrom-amount-string`.
3. Implement `TryFrom<&str>`, `TryFrom<String>`, and `TryFrom<Box<str>>` for `Amount` and `SignedAmount` in `units/src/amount/unsigned.rs` and `units/src/amount/signed.rs`.
4. Add unit tests for valid and invalid conversions.
5. Run `cargo test -p bitcoin-units`, `cargo fmt --check`, and `cargo clippy --all-targets --all-features -- -D warnings`.
6. Push and open a PR against `rust-bitcoin/rust-bitcoin:main`.
7. Update `WORK_DONE.md` with commands, diff, test output, and PR URL.

## Notes

- `Rc<str>` and `Arc<str>` were noted in the issue but are not implemented here because the `units` crate does not depend on `std::sync::Arc` or `alloc::rc::Rc`, and adding those dependencies would be out of scope for a minimal API-surface improvement.
- The `alloc`-gated implementations (`String`, `Box<str>`) preserve `no-std` compatibility.
- The `&str` implementation is always available.
