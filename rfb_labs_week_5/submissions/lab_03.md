# Lab 03 — 2-of-3 multisig wrapped in P2SH

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

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All four public tests in `tests/lab_03.rs` pass against `src/labs/lab03_p2sh.rs`:
`build_2_of_3_redeem_script` matches a `Builder`-constructed
`2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG` script exactly; `derive_p2sh_address` matches
`Address::p2sh(&script, network)`; `build_p2sh_script_pubkey` matches
`Address::p2sh(...).script_pubkey()`, i.e. `OP_HASH160 <scriptHash> OP_EQUAL`; and
`inspect_p2sh_multisig` reports an address starting with `2` (regtest P2SH prefix) and a
scriptPubKey starting with `a914` (`OP_HASH160` + 20-byte push).

## Explanation

P2SH validation happens in two layers, and satisfying the outer one proves nothing about the
inner one. The outer scriptPubKey — `OP_HASH160 <scriptHash> OP_EQUAL` — only checks that the
script you supply hashes (HASH160) to the exact value committed on-chain
(`build_p2sh_script_pubkey`/`derive_p2sh_address`). That is a pure hash-preimage check: *anyone*
who has seen the redeemScript on-chain (e.g. because it was already revealed in a previous spend,
or simply guessed correctly) can supply it and pass this first check, regardless of whether they
control any of the three keys inside it.

Passing the hash check only unlocks the *right* to have the redeemScript itself executed as the
real spending condition — in this case `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG`. That inner
script is what actually enforces "two of these three specific private keys must sign this exact
transaction." So matching the script hash proves you know *which* script governs this output; it
does not, by itself, satisfy that script's own rule. An attacker who reused a revealed redeemScript
without the required two signatures would pass the outer `OP_EQUAL` check and then immediately
fail `OP_CHECKMULTISIG`, so the transaction is still rejected overall. This is exactly why P2SH is
a *wrapper*, not a shortcut: it lets you commit to an arbitrarily complex spending policy behind a
short, fixed-size hash, while the policy itself is still fully enforced when the output is spent.
