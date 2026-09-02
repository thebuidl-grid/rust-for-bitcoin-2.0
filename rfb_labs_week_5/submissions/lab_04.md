# Lab 04 — Native P2WPKH

## Commands used

`cargo test --test lab_04`

## Terminal output

```bash
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.51s
     Running tests/lab_04.rs (target/debug/deps/lab_04-d3a7b49e00fd133b)

running 4 tests
test derives_a_native_regtest_address ... ok
test builds_a_version_zero_witness_lock ... ok
test leaves_scriptsig_empty_and_uses_witness ... ok
test reports_a_twenty_byte_program ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Code: src/labs/lab04_p2sg.rs  
Test: tests/lab_04.rs

## Explanation

### Why native P2WPKH has an empty ScriptSig.

Native SegWit (P2WPKH) has an empty scriptSig because the witness data, which contains the unlocking signature and public key, is moved out of the transaction's main body and into a separate structure called the Witness field.