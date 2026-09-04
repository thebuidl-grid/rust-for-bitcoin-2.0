# Lab 03 — P2SH 2-of-3 multisig

## Commands used

```
cargo test --test lab_03 -- --nocapture
cargo fmt --check
```

Implementation: `src/labs/lab03_p2sh.rs` — `build_2_of_3_redeem_script`,
`derive_p2sh_address`, `build_p2sh_script_pubkey`, `inspect_p2sh_multisig`.

## Terminal output

```
running 4 tests
test builds_the_outer_p2sh_lock ... ok
test derives_the_committed_p2sh_address ... ok
test builds_a_two_of_three_redeem_script ... ok
test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Observed behaviour for test keys `[1u8;32]`, `[2u8;32]`, `[3u8;32]`:
- redeemScript = `52 21<pub1> 21<pub2> 21<pub3> 53 ae`
  (`OP_2 <k1> <k2> <k3> OP_3 OP_CHECKMULTISIG`).
- `derive_p2sh_address` on regtest returns an address starting with `2`.
- `build_p2sh_script_pubkey` returns `a914<HASH160(redeemScript)>87`
  (`OP_HASH160 <push 20> OP_EQUAL`).
- `inspect_p2sh_multisig` ties all three together in one `P2shReport`.

## Evidence references

- `src/labs/lab03_p2sh.rs` — uses `Builder` + `OP_CHECKMULTISIG` and `ScriptBuf::new_p2sh`.
- `tests/lab_03.rs` — compares against a hand-built `Builder` script and `Address::p2sh`.
- Terminal block above from `cargo test --test lab_03`.

## Explanation

P2SH moves the spending policy off the output and behind a hash. The scriptPubKey
is only `OP_HASH160 <scriptHash> OP_EQUAL`, so the sender needs to know nothing
about the 2-of-3 rule — they just pay a `3...`/`2...` address. Spending happens in
two checks. First the *outer* check: the spender supplies the serialized
redeemScript as the last ScriptSig item, and the node verifies `HASH160(redeemScript)`
equals the committed `<scriptHash>`. Then the *inner* check: the redeemScript is
deserialized and executed as if it were the scriptPubKey, so `OP_CHECKMULTISIG`
runs against the signatures pushed earlier in the ScriptSig and requires any 2 of
the 3 listed keys to sign. `OP_CHECKMULTISIG` also consumes an extra dummy element
(the historical off-by-one bug), and signatures must appear in the same order as
their keys in the redeemScript.
