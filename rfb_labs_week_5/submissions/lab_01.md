# Lab 01 — Address and network identification

## Commands used

```
cargo test --test lab_01 -- --nocapture
```

## Terminal output

```
running 4 tests
test identifies_human_readable_prefixes ... ok
test maps_regtest_prefixes ... ok
test inspects_a_network_checked_address ... ok
test rejects_an_address_for_the_wrong_network ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Using the disposable test secret key `[0x01; 32]` on regtest, `Address::p2pkh` produced
`mrcNu71ztWjAQA6ww9kHiW3zBWSQidHXTQ`. Calling `inspect_address` on that string with
`Network::Regtest` returned:

```
AddressReport {
    address: "mrcNu71ztWjAQA6ww9kHiW3zBWSQidHXTQ",
    network: "regtest",
    format: P2pkh,
    script_pubkey_hex: "76a91479b000887626b294a914501a4cd226b58b23598388ac",
}
```

Re-checking the same address against `Network::Bitcoin` returned `Err`, confirming
`require_network` rejects a network mismatch even though the string itself parses fine.
`identify_prefix` correctly classified `1...` (P2PKH), `3...` (P2SH), `bc1q...` (P2WPKH),
and `bc1p...` (P2TR) using only the human-readable prefix.

## Explanation

The human-readable prefix (`1`, `3`, `bc1q`, `bc1p`, or their testnet/regtest equivalents)
is a strong hint about the script family an address encodes, because each Base58Check
version byte and each Bech32/Bech32m HRP + witness version is reserved for one address
type. But a prefix match is not sufficient proof of validity: Base58Check addresses carry
a 4-byte double-SHA256 checksum, and Bech32/Bech32m strings carry their own polymod
checksum — a single corrupted or mistyped character can still start with the right prefix
while being garbage underneath, so the checksum has to be verified before the payload is
trusted. Separately, the same payload format can be valid on more than one network (e.g.
regtest and testnet share the `m`/`n` P2PKH prefix), so a syntactically valid, checksum-
correct address can still be the wrong address for the network you intend to broadcast
to. That's why `inspect_address` performs three independent checks — parse, checksum
(inside parsing), and `require_network` — rather than trusting the prefix alone.
