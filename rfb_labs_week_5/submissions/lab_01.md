# Lab 01 — Address and network identification

## Commands used

```
cargo test --test lab_01 -- --nocapture
```

Ad-hoc check of `inspect_address` against a disposable regtest P2PKH address built from
the secp256k1 secret key `[1u8; 32]` (never a real key):

```rust
let regtest_pk = disposable_public_key(1);
let regtest_address = bitcoin::Address::p2pkh(regtest_pk, Network::Regtest);
let report = lab01_addresses::inspect_address(&regtest_address.to_string(), Network::Regtest).unwrap();
```

## Terminal output

```
running 4 tests
test identifies_human_readable_prefixes ... ok
test maps_regtest_prefixes ... ok
test rejects_an_address_for_the_wrong_network ... ok
test inspects_a_network_checked_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`inspect_address` on the disposable regtest address:

```
address = mrcNu71ztWjAQA6ww9kHiW3zBWSQidHXTQ
network = regtest
format = P2pkh
script_pubkey_hex = 76a91479b000887626b294a914501a4cd226b58b23598388ac
expected_prefix(P2wpkh, Regtest) = Some("bcrt1q")
```

Passing the same address to `inspect_address(..., Network::Bitcoin)` returns
`Err(WrongNetwork(...))`, confirming the network check runs even though the prefix
character (`m`/`n`) is only a hint, not proof.

## Evidence references

- `cargo test --test lab_01` output above, captured directly from this machine.
- Source: `src/labs/lab01_addresses.rs` (`identify_prefix`, `expected_prefix`,
  `inspect_address`, `script_pubkey_hex`).
- Test suite: `tests/lab_01.rs`.

## Explanation

Prefix inspection only tells you the *encoding family* a string claims to belong to
(`1`/`3` for Base58Check, `bc1q`/`bc1p` for Bech32/Bech32m). It cannot prove the string
is a valid address: a single mistyped character can still start with the right
character while failing the Base58Check checksum or the Bech32(m) checksum, and a
syntactically valid address can still belong to the wrong network, since legacy
testnet, regtest, and signet addresses share the same `m`/`n`/`2` prefixes as each
other. `identify_prefix` in Lab 01 is a cheap, first-pass classifier; `inspect_address`
does the real work by decoding the full string (rejecting a bad checksum) and then
calling `require_network`, which rejects an address that decodes fine but was minted
for a different chain. Skipping that second step is how funds get sent to a
correctly-shaped address on the wrong network.
