# Lab 01 — Address and network identification

## Commands used

```bash
cargo test --test lab_01
cargo fmt --check
```

## Terminal output

```text
$ cargo test --test lab_01
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running tests/lab_01.rs (target/debug/deps/lab_01-2e2b656970a529c5)

running 4 tests
test maps_regtest_prefixes ... ok
test identifies_human_readable_prefixes ... ok
test rejects_an_address_for_the_wrong_network ... ok
test inspects_a_network_checked_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo fmt --check` produced no diff.

## Evidence references

- Implementation: `src/labs/lab01_addresses.rs`
- Test suite: `tests/lab_01.rs`
- `identify_prefix` classifies by human-readable prefix alone (e.g. `1...` -> P2PKH,
  `3...`/`2...` -> P2SH, `bc1q.../tb1q.../bcrt1q...` -> P2WPKH, `bc1p.../tb1p.../bcrt1p...` ->
  P2TR) and correctly labels `bc1pexample`, a string that is not valid bech32m, purely from
  its prefix.
- `inspect_address` instead round-trips through `bitcoin::Address<NetworkUnchecked>::from_str`
  and `require_network`, so `rejects_an_address_for_the_wrong_network` fails a real regtest
  P2PKH address when checked against `Network::Bitcoin`, and
  `inspects_a_network_checked_address` confirms the returned `script_pubkey_hex` matches
  `address.script_pubkey().to_hex_string()` exactly.

## Explanation

Prefix inspection is a cheap heuristic, not proof of validity. A leading `1`, `3`, `bc1q`, or
`bc1p` tells you which encoding and script template an address is *supposed* to represent, but
it says nothing about whether the rest of the string is well-formed. A Base58Check address's
prefix is really just the first byte(s) of the payload after the version byte is prepended and
depends on the whole payload plus a 4-byte double-SHA256 checksum to catch typos and truncation;
a Bech32/Bech32m address depends on its BCH-code checksum over the full human-readable part and
data. `identify_prefix` in this lab deliberately only looks at the prefix (that's why it happily
labels the fake `bc1pexample` as P2TR), so it can be fooled by a mangled or truncated string.

`inspect_address` and `script_pubkey_hex` do the real work: they parse the full address through
`rust-bitcoin`, which validates the checksum and decodes the payload, and only then check that
the decoded network (mainnet/testnet/signet/regtest) matches the network the caller actually
intends to use. That second check matters independently of the checksum — Base58Check P2PKH
addresses on testnet and regtest happen to share the same version byte, so a syntactically valid,
checksum-correct address can still be valid on the *wrong* network. `require_network` is what
turns "well-formed" into "safe to pay," which is why `rejects_an_address_for_the_wrong_network`
must fail even though the address itself is perfectly valid — just for a different chain.

