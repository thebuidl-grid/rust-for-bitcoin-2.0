# Lab 03 — P2SH 2-of-3 multisig

## Commands used

```bash
cargo test --test lab_03
cargo fmt --check
```

## Terminal output

```text
$ cargo test --test lab_03
running 4 tests
test builds_the_outer_p2sh_lock ... ok
test derives_the_committed_p2sh_address ... ok
test builds_a_two_of_three_redeem_script ... ok
test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo fmt --check` produced no diff.

## Evidence references

- Implementation: `src/labs/lab03_p2sh.rs`
- Test suite: `tests/lab_03.rs`
- `build_2_of_3_redeem_script` reproduces `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG` byte-for-byte
  against a `bitcoin::script::Builder` built independently in the test.
- `derive_p2sh_address` and `build_p2sh_script_pubkey` HASH160 that redeemScript and match
  `Address::p2sh(&script, network)` exactly.
- `inspect_p2sh_multisig` combines all three and the test confirms `report.address` starts with
  `2` (regtest P2SH prefix) and `report.script_pubkey_hex` starts with `a914` (`OP_HASH160`
  push-20).

## Explanation

P2SH has two independent validation layers, and both must pass. The outer, on-chain layer is just
`OP_HASH160 <scriptHash> OP_EQUAL`: it checks that the redeemScript the spender reveals actually
hashes to the value committed by the address. That check is purely about *integrity* — it proves
"this is the script the sender intended to lock funds to," nothing more. It says nothing about
whether the script's own conditions are satisfied. The inner layer is the redeemScript itself —
here `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG` — which only runs after the hash check passes,
and it is what actually enforces the 2-of-3 signature rule. So matching the hash is necessary (a
wrong or malformed redeemScript is rejected immediately) but not sufficient: an attacker who
somehow reused a correct redeemScript still cannot spend without producing two valid signatures
from three permitted keys. This is exactly why P2SH could introduce complex scripts like multisig
without changing every wallet's address format: senders only ever need to understand "pay to this
hash," while the spending complexity is deferred entirely to the person redeeming the output.

