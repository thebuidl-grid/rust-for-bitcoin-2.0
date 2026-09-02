# Lab 01 — Address and network identification

## Commands used

`cargo test --test lab_01`

## Terminal output

```bash
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.01s
Running tests/lab_01.rs (target/debug/deps/lab_01-4fd37e4269b310ea)

running 4 tests
test identifies_human_readable_prefixes ... ok
test maps_regtest_prefixes ... ok
test inspects_a_network_checked_address ... ok
test rejects_an_address_for_the_wrong_network ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Code: src/labs/lab01_addresses.rs  
Test: tests/lab_01.rs

## Explanation

Prefix inspection only guesses the format based on the initial characteres, ignoring the rest of the string. Complete validation verifies the internal checksum to detect typos and ensures the characters belong to the correct alphabet. It also confirms the address is structurally valid and compatible with the specific network being used (Mainnet, Testnet, etc).
