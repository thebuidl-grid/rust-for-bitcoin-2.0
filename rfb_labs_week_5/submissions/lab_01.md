# Lab 01 — Address and network identification

## Commands used

```bash
cargo test --test lab_01
cargo fmt --check
bash grader/grade.sh
```

## Terminal output

```text
running 4 tests
test identifies_human_readable_prefixes ... ok
test maps_regtest_prefixes ... ok
test rejects_an_address_for_the_wrong_network ... ok
test inspects_a_network_checked_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Automated unit test suite in `tests/lab_01.rs` passing 4 unit tests covering human-readable prefix parsing (`1`, `3`, `bc1q`, `bc1p`, `m/n`, `2`, `bcrt1q`, `bcrt1p`), network validation (`require_network`), and scriptPubKey inspection.

## Explanation

Prefix inspection alone is not complete address validation because a human-readable prefix (such as `1`, `3`, or `bc1q`) only provides an initial visual hint about the intended address format and network. Complete address validation requires decoding the underlying encoding (Base58Check, Bech32, or Bech32m) to verify its checksum against typos or corruption, checking that the decoded payload length matches protocol specifications (such as a 20-byte HASH160 or 32-byte witness program), and strictly enforcing network safety so mainnet addresses are rejected on testnet/regtest environments to prevent accidental loss of funds.
