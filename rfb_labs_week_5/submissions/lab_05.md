# Lab 05 — Address compatibility map

## Commands used

`cargo test --test lab_05`

## Terminal output

```bash
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.37s
     Running tests/lab_05.rs (target/debug/deps/lab_05-eb8d8686bbdebb30)

running 4 tests
test builds_the_four_format_map ... ok
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok
test names_the_required_human_encoding ... ok
test selects_the_most_modern_supported_format ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
``` 

## Evidence references

Code: src/labs/lab05_compatibility.rs  
Test: tests/lab_05.rs

## Explanation

### Explain why a P2SH-era wallet may accept 3... but reject bc1q....

A `3...` address is a legacy P2SH address. Older wallets commonly support P2SH and Base58 encoding.  
A `bc1q...` address is a native SegWit address. It uses Bech32 encoding instead of Base58.  
Its script begins with `OP_0`, not `OP_HASH160`. Supporting it requires Bech32 and SegWit handling.  
A P2SH-era wallet may not recognize that format. It may therefore label `bc1q...` as invalid or unsupported.  