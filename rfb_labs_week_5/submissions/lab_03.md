# Lab 03 — P2SH 2-of-3 multisig

## Commands used

```bash
cargo test --test lab_03 -- --nocapture
cargo run --example labs_demo
```

## Terminal output

```text
$ cargo test --test lab_03 -- --nocapture
running 4 tests
test builds_the_outer_p2sh_lock ... ok
test reports_both_validation_layers ... ok
test derives_the_committed_p2sh_address ... ok
test builds_a_two_of_three_redeem_script ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
$ cargo run --example labs_demo   (Lab 03 section, disposable keys [1,2,3]u8;32)
inspect_p2sh_multisig = P2shReport {
    redeem_script_hex: "5221031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f
        21024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d07662102531fe606813450
        3d2723133227c867ac8fa6c83c537e9a44c3c5bdbdcb1fe33753ae",
    address: "2N99mC22Sz4sHLo6zSYkiCBmY47huZuMJbj",
    script_pubkey_hex: "a914ae79902ae33900b679c76ced8576362e4abb15e887" }
```

## Evidence references

- Implementation: [`src/labs/lab03_p2sh.rs`](../src/labs/lab03_p2sh.rs)
- Public test suite: [`tests/lab_03.rs`](../tests/lab_03.rs) — 4/4 passing, logged in
  [`grading/logs/lab_03.log`](../grading/logs/lab_03.log).
- `redeem_script_hex` starts with `52` (`OP_2`) and ends with `53ae` (`OP_3
  OP_CHECKMULTISIG`) around three 33-byte compressed pubkey pushes — the canonical
  `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG` layout.
- `script_pubkey_hex` starts with `a914` (`OP_HASH160` + 20-byte push) and ends with
  `87` (`OP_EQUAL`); the derived address starts with `2`, the regtest/testnet P2SH
  prefix.
- All three public keys are derived from disposable secrets `[1u8;32]`, `[2u8;32]`,
  `[3u8;32]` — never real keys.

## Explanation

P2SH (BIP16) splits validation into two independent layers, and matching the outer
layer is necessary but nowhere near sufficient to spend the coin:

1. **Outer layer — the scriptPubKey**, `OP_HASH160 <scriptHash> OP_EQUAL`. This only
   checks that the spender supplied *some* serialized script whose HASH160 equals the
   20-byte hash committed on-chain (`build_p2sh_script_pubkey`). Any script that hashes
   to the right value satisfies this check — the network has no idea, and does not
   care, what that script's rules actually are until this check passes.
2. **Inner layer — the redeemScript**, `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`
   (`build_2_of_3_redeem_script`). Once the outer hash check passes, the node executes
   the redeemScript itself against the remaining stack items. `OP_CHECKMULTISIG` then
   independently verifies that at least 2 of the 3 listed public keys have produced
   valid signatures over the spending transaction.

Because these two checks are sequential and independent, presenting the correct
redeemScript bytes (satisfying step 1) proves nothing about step 2: a spender who knows
the redeemScript's bytes but does not hold any of the three private keys can pass the
hash check and will still fail `OP_CHECKMULTISIG`, so the coin stays safe. The hash
commitment's only job is to let the *address itself* stay a fixed-size, 20-byte
Base58Check value regardless of how complex the actual spending policy behind it is —
the complexity (2-of-3, timelocks, etc.) lives entirely in the redeemScript, revealed
only when someone actually tries to spend.
