# Lab 01 — Address and network identification

## Commands used

```bash
cargo test --test lab_01 -- --nocapture
cargo run --example labs_demo
```

## Terminal output

```text
$ cargo test --test lab_01 -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s)
     Running tests/lab_01.rs (target/debug/deps/lab_01-de7a106bd9d48632)

running 4 tests
test maps_regtest_prefixes ... ok
test identifies_human_readable_prefixes ... ok
test inspects_a_network_checked_address ... ok
test rejects_an_address_for_the_wrong_network ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
$ cargo run --example labs_demo   (Lab 01 section)
identify_prefix("1BoatSLRHtKNngkdXEeobR76b53LETtpyT") = P2pkh
identify_prefix("3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy") = P2sh
identify_prefix("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080") = P2wpkh
identify_prefix("bc1pexample") = P2tr
expected_prefix(P2pkh, Regtest) = Some("m/n")
expected_prefix(P2sh, Regtest) = Some("2")
expected_prefix(P2wpkh, Regtest) = Some("bcrt1q")
expected_prefix(P2tr, Regtest) = Some("bcrt1p")
inspect_address(mrcNu71ztWjAQA6ww9kHiW3zBWSQidHXTQ, Regtest) =
  AddressReport { address: "mrcNu71ztWjAQA6ww9kHiW3zBWSQidHXTQ", network: "regtest",
                  format: P2pkh,
                  script_pubkey_hex: "76a91479b000887626b294a914501a4cd226b58b23598388ac" }
inspect_address(mrcNu71ztWjAQA6ww9kHiW3zBWSQidHXTQ, Bitcoin) =
  Err(WrongNetwork("validation error"))  (expected: Err)
```

## Evidence references

- Implementation: [`src/labs/lab01_addresses.rs`](../src/labs/lab01_addresses.rs)
- Public test suite: [`tests/lab_01.rs`](../tests/lab_01.rs) — 4/4 passing, captured verbatim
  in [`grading/logs/lab_01.log`](../grading/logs/lab_01.log) by `grader/grade.sh`.
- Reproducible demo: [`examples/labs_demo.rs`](../examples/labs_demo.rs), `lab01()` section.
- The regtest P2PKH address used above is derived from the disposable secret key
  `[1u8; 32]` (never a real key) purely so the demo has a real, parseable address to
  inspect.

## Explanation

A human-readable prefix (`1`, `3`, `bc1q`, `bc1p`, and their testnet/regtest
counterparts) is only a *hint* about which version byte or bech32 witness version an
address was encoded with — `identify_prefix` in this lab is nothing more than a string
match. It cannot tell us whether the address is actually well formed.

Two independent checks still have to happen before an address can be trusted, and
`inspect_address` performs both by going through `rust-bitcoin`'s
`Address<NetworkUnchecked>::from_str` and `require_network`:

1. **Checksum validation.** Base58Check addresses carry a 4-byte SHA256d checksum, and
   Bech32/Bech32m addresses carry a BCH-code checksum. A single mistyped or corrupted
   character is very likely to still start with the right prefix character, but it will
   fail checksum verification during parsing (`from_str` returns `Err`). Prefix
   inspection alone would happily accept a corrupted address that decoding rejects.
2. **Network validation.** The prefix does not fully pin down the network either — `m`
   and `n` are both used for testnet *and* regtest P2PKH, and nothing in the string
   itself proves the address was generated for the network the caller actually intends
   to use. `require_network` re-derives the address's real network from its decoded
   payload and errors out if it does not match the caller's expected network — which is
   exactly what `rejects_an_address_for_the_wrong_network` demonstrates: the same
   address string is accepted on `Regtest` and rejected on `Bitcoin`.

In short: the prefix is a fast, useful *first guess* for a UI to color-code or route an
address, but only full decode-plus-checksum-plus-network validation is safe to rely on
before treating an address as spendable or displaying it as verified.
