# Lab 01 — Address and network identification

## Commands used

```shell
test@pop-os:~/Desktop/rust/rust-for-bitcoin-2.0/rfb_labs_week_5$ cargo test --test lab_01
```

## Terminal output

```terminaloutput
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
Running tests/lab_01.rs (target/debug/deps/lab_01-ae29bb7080b78939)

running 4 tests
test identifies_human_readable_prefixes ... ok
test maps_regtest_prefixes ... ok
test inspects_a_network_checked_address ... ok
test rejects_an_address_for_the_wrong_network ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

```
Code: src/labs/lab01_addresses.rs  
Test: tests/lab_01.rs
```

## Explanation

Prefix inspection alone is **not** complete address validation. `identify_prefix`
only reads the human-readable characters that hint at the address family (for
example `bc1q` → P2WPKH, `1`/`m` → P2PKH). Three things it cannot catch:

1. **Encoding correctness.** A string beginning with `bc1q` might be malformed
   Base58Check or an invalid Bech32 string (bad checksum, mixed case, or wrong
   length). The prefix says nothing about whether the payload round-trips.

2. **Network binding.** `m`/`n` and `bcrt1q` only exist on regtest/testnet, while
   `1` and `bc1q` belong to mainnet. A check that looks only at the leading
   character cannot confirm the address belongs to the network the caller expects.

3. **The real payload.** The scriptPubKey is derived from the cryptographic
   hash *after* the prefix, not from the prefix itself. Only a full decode lets us
   verify the format, extract the 20/32-byte program, and build the locking script.

This is why `inspect_address` goes beyond the prefix: it runs the string through
`Address::from_str` (which checksum-validates the encoding) and then `require_network`
(which rejects addresses bound to another chain) before building an `AddressReport`
that carries the decoded `script_pubkey_hex`. A bare prefix check could happily
accept an address that fails checksum verification or targets the wrong network.

