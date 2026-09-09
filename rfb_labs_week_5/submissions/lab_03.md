# Lab 03 — P2SH 2-of-3 multisig

## Commands used

```bash
cargo test --test lab_03
bash grader/grade.sh
```

## Terminal output

running 4 tests

test builds_a_two_of_three_redeem_script ... ok

test builds_the_outer_p2sh_lock ... ok

test derives_the_committed_p2sh_address ... ok

test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

## Evidence references

All four public tests in `tests/lab_03.rs` pass, covering:
- Building the canonical `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG` redeem
  script (`builds_a_two_of_three_redeem_script`)
- Deriving the P2SH address that commits to that redeem script's HASH160
  (`derives_the_committed_p2sh_address`)
- Building the outer `OP_HASH160 <scriptHash> OP_EQUAL` scriptPubKey
  (`builds_the_outer_p2sh_lock`)
- Reporting all three values together and confirming the regtest P2SH
  address prefix (`2`) and outer scriptPubKey opcode prefix (`a914`)
  (`reports_both_validation_layers`)

## Explanation


P2SH validation happens in two separate layers, and satisfying the outer
layer alone proves nothing about the inner one.

The **outer check** — `OP_HASH160 <scriptHash> OP_EQUAL` — only verifies
that the redeemScript supplied by the spender hashes (HASH160) to the same
20-byte value committed in the P2SH output when it was created. This is a
simple data-integrity check: it confirms "this is the same script the
sender locked funds to," but it says nothing about whether the spender is
actually authorized to use that script.

The **inner check** is the redeemScript itself, which only runs *after* the
hash matches. Here it's `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG` — this
is where real authorization is enforced: the spender must supply at least
two valid signatures, each corresponding to two of the three listed public
keys, or `OP_CHECKMULTISIG` fails and the whole spend is rejected.

So matching the script hash only proves "you know which redeemScript was
committed to" — anyone can read a P2SH address's committed hash from the
blockchain and could in principle guess or copy the correct redeemScript
bytes. What actually protects the funds is that even after supplying the
correct redeemScript, the spender still has to satisfy that script's own
rules — in this case, producing two valid multisig signatures. The outer
hash check is a commitment/integrity check; the inner script is the real
spending policy.

