# Lab 03 — P2SH 2-of-3 multisig

## Commands used

```bash
cargo test --test lab_03 -- --nocapture
```

## Terminal output

```
running 4 tests
test builds_the_outer_p2sh_lock ... ok
test derives_the_committed_p2sh_address ... ok
test builds_a_two_of_three_redeem_script ... ok
test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Evidence references

`src/labs/lab03_p2sh.rs`:

- `build_2_of_3_redeem_script` uses `bitcoin::script::Builder` to push `OP_2`, the three
  public keys, `OP_3`, then `OP_CHECKMULTISIG` — the canonical 2-of-3
  multisig script.
- `derive_p2sh_address` HASH160s that redeemScript via `Address::p2sh(&script, network)`.
- `build_p2sh_script_pubkey` returns the outer lock,
  `ScriptBuf::new_p2sh(&script.script_hash())`, which is just
  `OP_HASH160 <scriptHash> OP_EQUAL` — confirmed in the test by asserting the hex starts
  with `a914` (`OP_HASH160` + push-20).
- `inspect_p2sh_multisig` combines all three into a `P2shReport`; the regtest test
  asserts the resulting address starts with `2`, the P2SH regtest prefix.

## Explanation

Two layers, two different jobs. The outer scriptPubKey — `OP_HASH160 <scriptHash>
OP_EQUAL` — only checks that whoever's spending can produce a script whose hash
matches what's committed on-chain. That's necessary (it's the whole reason a sender
can pay a short fixed-size address without ever seeing the multisig setup), but it
isn't remotely sufficient. Once a P2SH output has been spent once, the redeemScript
is sitting in the chain's history for anyone to read — so passing the outer hash
check again on a later output using the same script proves nothing about key
ownership either.

The actual spending rule lives one layer down. Once the hash matches, the
redeemScript itself gets pushed and run as the real scriptPubKey:
`2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`, which demands two valid signatures out of
the three named keys against the real spending transaction. So matching the outer
hash proves you know the correct script text. It says nothing about whether you hold
any of the three private keys — that's entirely down to OP_CHECKMULTISIG.
