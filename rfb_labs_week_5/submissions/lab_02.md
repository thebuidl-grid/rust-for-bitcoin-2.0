# Lab 02 — Legacy P2PKH

## Commands used

`cargo test --test lab_02`

## Terminal output

```bash
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.27s
Running tests/lab_02.rs (target/debug/deps/lab_02-1d64efbce3d4f4b7)

running 4 tests
test puts_unlocking_data_in_scriptsig ... ok
test builds_the_standard_p2pkh_lock ... ok
test commits_to_hash160_of_the_public_key ... ok
test derives_the_expected_p2pkh_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Code: src/labs/lab02_p2pkh.rs  
Test: tests/lab_02.rs

## Explanation

### P2PKH locking and unlocking

P2PKH locks funds by creating a script that requires the spender to provide a public key and a valid signature. The lock stores only a HASH160 hash of the public key to save space on the blockchain. To unlock it, the spender provides their full public key and a signature; the network then hashes the provided key to see if it matches the stored hash and verifies the signature against that key.

