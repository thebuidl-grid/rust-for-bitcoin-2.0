# Lab 03 — P2SH 2-of-3 multisig

## Commands used

```bash
cargo test --test lab_03
cargo fmt --check
bash grader/grade.sh
```

## Terminal output

```text
running 4 tests
test builds_the_outer_p2sh_lock ... ok
test derives_the_committed_p2sh_address ... ok
test builds_a_two_of_three_redeem_script ... ok
test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Automated unit tests in `tests/lab_03.rs` passing 4 checks that construct a canonical 2-of-3 multisig redeemScript (`2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`), derive its committed P2SH address (`2...` on Regtest / `3...` on Mainnet), and build the outer locking script (`OP_HASH160 <scriptHash> OP_EQUAL`).

## Explanation

P2SH splits spending validation into two distinct execution layers. The outer layer scriptPubKey (`OP_HASH160 <scriptHash> OP_EQUAL`) verifies only that the HASH160 of the push-data `redeemScript` in `ScriptSig` matches the 20-byte script hash stored in the UTXO. Matching this hash is necessary to reveal the script, but does not itself authorize the spend. Once the hash check passes, BIP16 causes the Bitcoin script interpreter to deserialize and execute the inner `redeemScript` (`2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`) against the signatures remaining on the stack, ensuring that the 2-of-3 multisig policy is enforced.
