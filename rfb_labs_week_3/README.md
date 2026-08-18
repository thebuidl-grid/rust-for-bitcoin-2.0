# Rust for Bitcoin 2.0 — Week 3

Contribute to the rust-bitcoin open-source project by implementing a real
issue and opening a pull request.

## Recommended workflow

1. Read [ASSIGNMENT.md](ASSIGNMENT.md).
2. Fork and clone `rust-bitcoin/rust-bitcoin`.
3. Create a feature branch `tryfrom-amount-string`.
4. Implement `TryFrom` conversions for `Amount` and `SignedAmount`.
5. Add tests in the `units` crate.
6. Run `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`.
7. Push and open a PR against `main`.
8. Document your work in `WORK_DONE.md`.

```bash
# In your rust-bitcoin fork
git clone https://github.com/<your-username>/rust-bitcoin.git
cd rust-bitcoin
git checkout -b tryfrom-amount-string

# Implement changes in units/src/amount/unsigned.rs and units/src/amount/signed.rs

cargo test -p bitcoin-units
cargo fmt --check -p bitcoin-units
cargo clippy --all-targets --all-features -- -D warnings -p bitcoin-units

git add units/src/amount/unsigned.rs units/src/amount/signed.rs
git commit -m "feat(units): add TryFrom string conversions for Amount and SignedAmount"
git push origin tryfrom-amount-string
```

## Pull-request checklist

- [ ] PR title follows the repository's conventional-commit style.
- [ ] PR description links the assigned issue and summarises the change.
- [ ] All tests pass in the `units` crate.
- [ ] `cargo fmt --check` passes.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [ ] `WORK_DONE.md` in this folder is completed with evidence.
