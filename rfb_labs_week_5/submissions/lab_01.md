# Lab 01 — Address and network identification

## Commands used

```bash
cargo test --test lab_01 -- --nocapture
```

## Terminal output

```
running 4 tests
test maps_regtest_prefixes ... ok
test identifies_human_readable_prefixes ... ok
test rejects_an_address_for_the_wrong_network ... ok
test inspects_a_network_checked_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All four public tests in `tests/lab_01.rs` pass against `src/labs/lab01_addresses.rs`:
`identify_prefix` maps `1...`/`3...`/`bc1q...`/`bc1p...` to `P2pkh`/`P2sh`/`P2wpkh`/`P2tr`;
`expected_prefix` reproduces the regtest table (`m/n`, `2`, `bcrt1q`, `bcrt1p`);
`inspect_address` parses a regtest P2PKH address, reports `network: "regtest"` and the correct
`script_pubkey_hex`; `rejects_an_address_for_the_wrong_network` confirms `inspect_address` and
`script_pubkey_hex` both return `Err` when a regtest address is checked against `Network::Bitcoin`.

## Explanation

A human-readable prefix (`1`, `3`, `bc1q`, `bc1p`, ...) is only a *convention*, not a proof. It is
useful for a quick guess at the script type, but it is not itself validated by anything — a
malformed or truncated string could still start with `1` without being a real address. Two
independent checks are required before an address can be trusted:

1. **Checksum validation.** Base58Check addresses carry a 4-byte double-SHA256 checksum, and
   Bech32/Bech32m addresses carry a checksum baked into their encoding. `Address::from_str`
   recomputes and verifies this checksum while parsing — if a single character was mistyped or
   corrupted, parsing fails with an error rather than silently accepting a bad address. This is
   what `identify_prefix` in this lab does *not* do (it is pure string inspection), which is why
   `inspect_address` exists as a separate, real parsing/validation path.
2. **Network validation.** The same script type can be encoded differently on different networks
   (e.g. mainnet P2PKH starts with `1`, regtest/testnet P2PKH starts with `m`/`n`), and Base58 in
   particular does not encode a *distinct* value per network the way Bech32's HRP does as
   obviously — so parsing alone doesn't guarantee the address belongs to the network you intend to
   send funds to. `Address::require_network` performs this second, separate check, and
   `rejects_an_address_for_the_wrong_network` proves that a regtest address is correctly rejected
   when checked against `Network::Bitcoin`. Skipping this step is a classic way to construct a
   transaction that is valid Bitcoin but pays into the wrong network's address space.
