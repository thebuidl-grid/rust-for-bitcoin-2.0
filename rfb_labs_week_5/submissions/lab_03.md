# Lab 03 — P2SH 2-of-3 multisig

## Commands used

```bash
cargo test --test lab_03 -- --nocapture
cargo fmt --check
cargo clippy --all-targets
```

Implementation lives in `src/labs/lab03_p2sh.rs`: `build_2_of_3_redeem_script`,
`derive_p2sh_address`, `build_p2sh_script_pubkey`, and
`inspect_p2sh_multisig`.

## Terminal output

```
running 4 tests
test builds_the_outer_p2sh_lock ... ok
test derives_the_committed_p2sh_address ... ok
test builds_a_two_of_three_redeem_script ... ok
test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

- `src/labs/lab03_p2sh.rs` — the `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`
  redeemScript, its HASH160 commitment, and the outer `OP_HASH160 <hash>
  OP_EQUAL` scriptPubKey.
- `tests/lab_03.rs` — `reports_both_validation_layers` asserts the address
  starts with `2` (regtest P2SH prefix) and the scriptPubKey hex starts with
  `a914` (`OP_HASH160` + a 20-byte push).
- `bash grader/grade.sh` recorded `03 | 4/4 | 4 | ...` for this lab.

## Explanation

P2SH validation happens in two independent layers, and passing the first
layer proves nothing about the second. The outer scriptPubKey
(`OP_HASH160 <scriptHash> OP_EQUAL`) only commits to the HASH160 of the
redeemScript; a spender satisfies it by revealing any byte string that hashes
to `scriptHash` and pushing it as the last stack item — that is a hash-preimage
check, not a signature check. Once that first layer passes, the node takes the
very same bytes, deserializes them as a script, and executes *that* script
(here the 2-of-3 `OP_CHECKMULTISIG` rule) against whatever unlocking data
precedes it in the ScriptSig.

So matching the script hash only proves the spender knows the correct
redeemScript bytes — it says nothing about whether they can also satisfy the
rule those bytes encode. `OP_CHECKMULTISIG` still has to run and verify that at
least 2 of the 3 committed public keys produced valid signatures over the
transaction. `inspect_p2sh_multisig`'s `redeem_script_hex` is exactly the
program that layer two executes; without two genuine signatures, presenting
the correct redeemScript satisfies `OP_EQUAL` but the multisig check still
fails and the whole script evaluates to false, so the P2SH commitment being
correct is necessary but not sufficient to spend the coin.
