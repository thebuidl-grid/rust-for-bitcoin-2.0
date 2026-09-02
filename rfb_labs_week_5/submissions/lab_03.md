# Lab 03 — P2SH 2-of-3 multisig

## Commands used

`cargo test --test lab_03`

## Terminal output

```bash
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/lab_03.rs (target/debug/deps/lab_03-3df1e926531d5f77)

running 4 tests
test builds_the_outer_p2sh_lock ... ok
test derives_the_committed_p2sh_address ... ok
test builds_a_two_of_three_redeem_script ... ok
test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Code: src/labs/lab03_p2sg.rs  
Test: tests/lab_03.rs

## Explanation

Outer hash check and inner multisig check.

A P2SH output contains:

`OP_HASH160 <redeem_script_hash> OP_EQUAL`

The redeem script is the embedded 2-of-3 multisig rule. When spending, the input provides the signatures and redeem script.
Bitcoin hashes the provided redeem script with HASH160. It compares that hash with the hash stored in the output.
If the hashes differ, the spend immediately fails. This proves that the correct redeem script was supplied.
Bitcoin then executes the redeem script itself:

`2 <pubkey1> <pubkey2> <pubkey3> 3 OP_CHECKMULTISIG`

At least two of the three signatures must be valid. Each signature must authorize the transaction being spent.
The outer check verifies the script, while the inner check verifies authorization.
