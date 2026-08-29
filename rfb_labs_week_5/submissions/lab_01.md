# Lab 01 — Address and network identification

## Commands used

<!-- TODO: List the Rust commands you ran. -->
```bash
xoulomon@xoulomon-ThinkPad-L13-Gen-2a:~/Desktop/rust4BTC/rust-for-bitcoin-2.0/rfb_labs_week_5$ cargo test --test lab_01
```

## Terminal output

<!-- TODO: Record the checked formats, networks, and scriptPubKeys. -->
```bash
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running tests/lab_01.rs (target/debug/deps/lab_01-8ced939ae037b757)

running 4 tests
test identifies_human_readable_prefixes ... ok
test maps_regtest_prefixes ... ok
test inspects_a_network_checked_address ... ok
test rejects_an_address_for_the_wrong_network ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

xoulomon@xoulomon-ThinkPad-L13-Gen-2a:~/Desktop/rust4BTC/rust-for-bitcoin-2.0/rfb_labs_week_5$ 
```

## Evidence references

TODO: Link screenshots or describe attached evidence.

## Explanation

<!-- TODO: Explain why prefix inspection alone is not complete address validation. -->

- Typos/corruption — a single mistyped character deeper in the address still "looks" like a valid prefix, but the checksum (built into Base58Check/Bech32) is what actually catches this. Prefix matching skips the checksum entirely.
- Wrong network — "bc1pexample" starts with bc1p, so identify_prefix happily calls it P2TR, even though it isn't real bech32m data at all. A prefix can't tell you the address decodes correctly or that it belongs to the network you intend to use (mainnet vs. testnet vs. regtest).
- Garbage after the prefix — nothing stops junk characters from following a valid-looking prefix.

That's why inspect_address/script_pubkey_hex don't rely on identify_prefix — they call Address::from_str (which validates the checksum/encoding) and require_network (which enforces the network) before trusting anything about the address. The prefix is just a cheap first guess; real validation requires actually parsing it.

