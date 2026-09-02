# Lab 03 — P2SH 2-of-3 multisig

## Commands used

```shell
test@pop-os:~/Desktop/rust/rust-for-bitcoin-2.0/rfb_labs_week_5$ cargo test --test lab_03
```

## Terminal output

```terminaloutput
running 4 tests
test builds_the_outer_p2sh_lock ... ok
test reports_both_validation_layers ... ok
test derives_the_committed_p2sh_address ... ok
test builds_a_two_of_three_redeem_script ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The redeemScript built by `build_2_of_3_redeem_script` is the canonical
`2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`. Its HASH160 is committed by the P2SH
address (regtest prefix `2`), and the outer locking script built by
`build_p2sh_script_pubkey` is `OP_HASH160 <scriptHash> OP_EQUAL` (hex prefix
`a914`).

## Evidence references

```
Code: src/labs/lab03_p2sh.rs
Test: tests/lab_03.rs
```

## Explanation

P2SH (Pay-to-Script-Hash) has **two validation layers**.

**Outer hash check.** The scriptPubKey in the output is only
`OP_HASH160 <scriptHash> OP_EQUAL`. On the chain, the coins are committed to the
HASH160 (hash256-style `RIPEMD160(SHA256)`) of the serialized redeemScript — not to
the script itself. When spending, the spender must reveal the full redeemScript whose
hash equals the committed value. This check only proves that the presented script is
*the one the output committed to*; by itself it authorizes nothing.

**Inner multisig check.** The revealed redeemScript is then executed as a script in
its own right. For this lab it is a 2-of-3 multisig:
`2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`. The inner check verifies that a valid
signature set from at least 2 of the 3 listed public keys is present. Only passing
this inner rule actually releases the funds.

Matching the script hash is therefore **necessary but not sufficient**: it confirms
the redeemScript is the committed one, but it does not prove that the required
multisig signatures exist. The coins only move once both the outer hash matches and
the inner 2-of-3 rule is satisfied.
