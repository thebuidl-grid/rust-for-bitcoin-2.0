# Lab 01 — Address and network identification

## Commands used

```bash
cargo test --test lab_01 -- --nocapture
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## Terminal output

```
running 4 tests
test identifies_human_readable_prefixes ... ok
test maps_regtest_prefixes ... ok
test inspects_a_network_checked_address ... ok
test rejects_an_address_for_the_wrong_network ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Evidence references

`src/labs/lab01_addresses.rs` implements all four functions:

- `identify_prefix` reads only the leading characters of the address string
(`1`, `3`/`2`, `bc1q`/`bcrt1q`/`tb1q`, `bc1p`/`bcrt1p`/`tb1p`) and returns the
matching `AddressFormat`, with no parsing or checksum work at all.
- `expected_prefix` maps `(AddressFormat, Network)` to the human-readable prefix a
correctly encoded address on that network must start with, e.g. `Regtest` +
`P2wpkh` → `"bcrt1q"`.
- `inspect_address` calls `bitcoin::Address::from_str(address)?.require_network(network)?`
before doing anything else, then reads `address.address_type()` to build an
`AddressReport` carrying the checked address string, lowercase network name, format,
and `scriptPubKey` hex.
- `script_pubkey_hex` reuses `inspect_address` and returns just the scriptPubKey field.

`tests/lab_01.rs::rejects_an_address_for_the_wrong_network` proves the network check is
enforced: a regtest P2PKH address passed with `Network::Bitcoin` returns `Err(..)` from
both `inspect_address` and `script_pubkey_hex`.

## Explanation

`identify_prefix` is a guess, nothing more — it looks at the first couple of
characters and stops. It can't tell you if the rest of the string is even valid
base58 or bech32, let alone whether it belongs to the network you think you're on.

That's the whole reason `inspect_address` doesn't just do a prefix check and call it
done. Base58Check addresses carry a 4-byte double-SHA256 checksum; Bech32/Bech32m
carries its own polynomial checksum over the HRP and data. Either one catches a typo
or a dropped character that a prefix match would happily let through. But checksum
validity and network correctness are two separate questions — a mainnet P2PKH and a
testnet P2PKH differ only in their version byte, so a perfectly checksummed address
can still be for the wrong chain entirely. `from_str` proves the checksum is good.
`require_network` is the part that actually stops you from paying a mainnet address
while your wallet thinks it's on testnet (or the reverse) — that's a real class of
bug, not a hypothetical one.