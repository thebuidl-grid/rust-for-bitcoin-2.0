# Lab 01 — Address and network identification

## Commands used

```
cargo test --test lab_01 -- --nocapture
cargo fmt --check
```

Implementation lives in `src/labs/lab01_addresses.rs`. The functions exercised are
`identify_prefix`, `expected_prefix`, `inspect_address`, and `script_pubkey_hex`.

## Terminal output

```
running 4 tests
test maps_regtest_prefixes ... ok
test identifies_human_readable_prefixes ... ok
test rejects_an_address_for_the_wrong_network ... ok
test inspects_a_network_checked_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Observed behaviour:
- `1BoatSLRHtKNngkdXEeobR76b53LETtpyT` -> P2PKH, `3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy` -> P2SH,
  `bc1q...` -> P2WPKH, `bc1p...` -> P2TR.
- A regtest P2PKH address inspected on `Network::Regtest` returns network `"regtest"`,
  format `P2pkh`, and `script_pubkey_hex` equal to `76a914<20-byte-hash>88ac`.
- The same address inspected on `Network::Bitcoin` returns `Err(LabError::WrongNetwork(..))`,
  and `script_pubkey_hex` fails the same way.

## Evidence references

- `src/labs/lab01_addresses.rs` — full implementation with helper `checked_address`.
- `tests/lab_01.rs` — the public assertions reproduced by the run above.
- Terminal output block in this file is copied verbatim from `cargo test --test lab_01`.

## Explanation

A prefix only tells you which *encoding family* an address claims to belong to:
`1`/`m`/`n` and `3`/`2` are Base58Check version bytes, while `bc1`/`tb1`/`bcrt1`
are bech32/bech32m human-readable parts whose first data character encodes the
witness version. Reading the prefix cannot confirm that the Base58 checksum is
valid, that the bech32 checksum and character set are correct, that the payload is
the right length (20 vs 32 bytes), or that the network matches the one you intend
to spend on. `inspect_address` therefore parses into `Address<NetworkUnchecked>`,
calls `require_network`, and only then reads `address_type()` and `script_pubkey()`.
The wrong-network test shows why: a well-formed mainnet-looking string must still be
rejected when the caller asked for regtest, because paying it would burn funds on the
wrong chain. Prefix inspection is a fast triage step; real validation is checksum +
length + network + script-type derivation.
