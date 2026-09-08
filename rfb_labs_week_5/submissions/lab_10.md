# Lab 10 — Deterministic recovery across address families

## Commands used

```bash
cargo test --test lab_10
cargo fmt --check
bash grader/grade.sh
```

## Terminal output

```text
running 4 tests
test identical_recovery_inputs_repeat ... ok
test changing_only_the_index_changes_the_address ... ok
test format_selection_changes_the_lock_target ... ok
test derives_three_regtest_address_families ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

## Evidence references

Automated unit tests in `tests/lab_10.rs` verifying multi-family address derivation across BIP44 P2PKH (`m/n...`), BIP49 P2SH-P2WPKH (`2...`), and BIP84 P2WPKH (`bcrt1q...`), deterministic repeatability, child index sensitivity, and script target locking differences.

## Explanation

Deterministic wallet recovery works because all child keys and addresses are mathematically derived from a single master seed using deterministic hashing functions (HMAC-SHA512). When provided with identical recovery inputs (mnemonic words, optional passphrase, derivation path, network, and script format), the wallet produces the exact same private key and address every time.

However, restoring a wallet successfully also requires adhering to derivation path and scriptPubKey standards:
- **BIP44** (`m/44'/coin'/account'/change/index`) -> Legacy P2PKH (`1...` / `m/n...`)
- **BIP49** (`m/49'/coin'/account'/change/index`) -> Nested SegWit P2SH-P2WPKH (`3...` / `2...`)
- **BIP84** (`m/84'/coin'/account'/change/index`) -> Native SegWit P2WPKH (`bc1q...` / `bcrt1q...`)

If wallet software restores a mnemonic using the wrong path standard (for example, checking BIP44 when funds were stored on a BIP84 branch), it will derive different addresses and display a zero balance. Successful recovery requires both the correct secret recovery root and the correct path/script conventions.
