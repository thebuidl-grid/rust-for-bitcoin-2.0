# Lab 01 — Address and network identification

## Commands used

I formatted the Rust source and ran both focused tests and the complete Lab 1 test
suite:

```bash
cargo fmt
cargo test --test lab_01 identifies_human_readable_prefixes
cargo test --test lab_01 maps_regtest_prefixes
cargo test --test lab_01 inspects_a_network_checked_address
cargo test --test lab_01 rejects_an_address_for_the_wrong_network
cargo test --test lab_01
```

## Terminal output

The complete Lab 1 test suite passed all four public tests:

```text
running 4 tests
test identifies_human_readable_prefixes ... ok
test maps_regtest_prefixes ... ok
test rejects_an_address_for_the_wrong_network ... ok
test inspects_a_network_checked_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The tests confirmed prefix classification, regtest prefix mapping, address parsing,
network enforcement, and scriptPubKey extraction.

## Evidence references

- Implementation: `src/labs/lab01_addresses.rs`
- Public test suite: `tests/lab_01.rs`
- `inspect_address` parses an `Address<NetworkUnchecked>`, calls
  `require_network`, maps the validated address type, and records the scriptPubKey
  as hexadecimal.
- `script_pubkey_hex` reuses `inspect_address`, preserving its parsing and network
  checks.

## Explanation

<!-- Write your explanation here. Explain why a prefix is only a format clue and why
checksum, encoding, and network validation are still required. -->
