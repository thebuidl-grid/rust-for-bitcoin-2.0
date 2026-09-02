# Lab 03 — P2SH 2-of-3 multisig

**Author:** [Christopher Dominic Eze](https://github.com/Christopherdominic)

## Commands used

```bash
cargo test --test lab_03
cargo run --example evidence   # scratch script, deleted after copying the output below
```

## Terminal output

```
running 4 tests
test builds_the_outer_p2sh_lock ... ok
test builds_a_two_of_three_redeem_script ... ok
test derives_the_committed_p2sh_address ... ok
test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Using the three disposable test keys from `[1u8;32]`, `[2u8;32]`, `[3u8;32]`:

```
redeemScript:  5221031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f21024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d07662102531fe6068134503d2723133227c867ac8fa6c83c537e9a44c3c5bdbdcb1fe33753ae
P2SH address:  2N99mC22Sz4sHLo6zSYkiCBmY47huZuMJbj
scriptPubKey:  a914ae79902ae33900b679c76ced8576362e4abb15e887
```

## Evidence references

- `src/labs/lab03_p2sh.rs` — `build_2_of_3_redeem_script`, `derive_p2sh_address`,
  `build_p2sh_script_pubkey`, `inspect_p2sh_multisig`.
- `tests/lab_03.rs::reports_both_validation_layers` — checks the address starts with
  `2` (regtest P2SH prefix) and the scriptPubKey starts with `a914`, both true above.
- Reading the redeemScript hex: `52` = `OP_2`, then three `21`-prefixed (33-byte)
  compressed pubkeys, then `53` = `OP_3`, then `ae` = `OP_CHECKMULTISIG` — the
  canonical 2-of-3 script.
- Reading the scriptPubKey: `a914` (`OP_HASH160`, push 20 bytes) +
  `ae79902ae33900b679c76ced8576362e4abb15e` + `87` (`OP_EQUAL`).

## Explanation

P2SH is a two-layer construction, and the two layers check completely different
things. The outer scriptPubKey — `OP_HASH160 <scriptHash> OP_EQUAL` — only checks one
thing: does `HASH160` of whatever redeemScript the spender supplies equal the 20-byte
hash committed on-chain? That's a cheap, generic check that doesn't know or care
whether the redeemScript is a 2-of-3 multisig, a timelock, or anything else. It's
matching a fingerprint, nothing more.

Passing that check only proves the spender knows *a* script whose hash matches — it
says nothing about whether the conditions inside that script are satisfied. That's
what the second layer is for. Once the outer hash check passes, the node takes the
redeemScript the spender just revealed and executes it against whatever else the
spender put on the stack (in this case, two signatures, because it's 2-of-3). So the
inner check is `OP_CHECKMULTISIG` actually verifying two valid signatures from two of
the three committed public keys, against the spending transaction.

So matching the hash is necessary — you can't skip it, since it's what proves you're
using the *right* redeemScript rather than a substitute one — but it's not
sufficient by itself. Someone could reveal the correct redeemScript and still fail to
spend if they can't produce two valid signatures for it. Both layers have to pass:
outer hash match proves "this is the script that was committed to," inner script
execution proves "the spending conditions in that script are actually met." That
separation is also what makes P2SH generic — the outer layer never needs to know what
kind of script it's wrapping, which is exactly how it can wrap multisig, or any other
script, without Bitcoin needing a dedicated address type for each one.
