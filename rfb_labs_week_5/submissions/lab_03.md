# Lab 03 — P2SH 2-of-3 multisig

## Commands used

```
cargo test --test lab_03 -- --nocapture
```

Ad-hoc check using three disposable public keys derived from secp256k1 secret keys
`[1u8; 32]`, `[2u8; 32]`, `[3u8; 32]`:

```rust
let keys = [disposable_public_key(1), disposable_public_key(2), disposable_public_key(3)];
lab03_p2sh::inspect_p2sh_multisig(key_refs, Network::Regtest)
```

## Terminal output

```
running 4 tests
test builds_the_outer_p2sh_lock ... ok
test builds_a_two_of_three_redeem_script ... ok
test derives_the_committed_p2sh_address ... ok
test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```
redeem_script = 5221031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f21024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d07662102531fe6068134503d2723133227c867ac8fa6c83c537e9a44c3c5bdbdcb1fe33753ae
p2sh address = 2N99mC22Sz4sHLo6zSYkiCBmY47huZuMJbj
script_pubkey = a914ae79902ae33900b679c76ced8576362e4abb15e887
```

## Evidence references

- `cargo test --test lab_03` output above.
- Source: `src/labs/lab03_p2sh.rs`.
- Test suite: `tests/lab_03.rs`.

## Explanation

P2SH splits validation into two independent layers. The outer scriptPubKey
(`OP_HASH160 <scriptHash> OP_EQUAL`) only checks that the redeemScript supplied at
spend time hashes to the committed `scriptHash` — it says nothing about what that
script requires. Matching the hash proves you supplied *the correct script*, not that
you are *entitled to spend*. The inner rule, `2 <pub1> <pub2> <pub3> 3
OP_CHECKMULTISIG`, is only evaluated once the outer hash check passes, and it demands
two valid signatures from the three named keys. A spender who supplies the right
redeemScript but cannot produce two of the three signatures still fails at the inner
check. This is why the two layers are described separately in the `P2shReport`: the
scriptHash proves the script wasn't tampered with, and OP_CHECKMULTISIG proves the
spender actually satisfies the multisig policy encoded inside it.
