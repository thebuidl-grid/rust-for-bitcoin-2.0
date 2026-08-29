# Lab 03 — P2SH 2-of-3 multisig

## Commands used

```
cargo test --test lab_03 -- --nocapture
```

## Terminal output

```
running 4 tests
test derives_the_committed_p2sh_address ... ok
test builds_the_outer_p2sh_lock ... ok
test builds_a_two_of_three_redeem_script ... ok
test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Using the three disposable test secret keys `[0x01; 32]`, `[0x02; 32]`, `[0x03; 32]` on
regtest:

```
redeemScript:       5221031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f
                    21024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d07662
                    102531fe6068134503d2723133227c867ac8fa6c83c537e9a44c3c5bdbdcb1fe33753ae
P2SH address:       2N99mC22Sz4sHLo6zSYkiCBmY47huZuMJbj
outer scriptPubKey: a914ae79902ae33900b679c76ced8576362e4abb15e887
```

The redeemScript decodes as `OP_2 <pub1> <pub2> <pub3> OP_3 OP_CHECKMULTISIG`. The outer
scriptPubKey is `OP_HASH160 <20-byte HASH160(redeemScript)> OP_EQUAL`, and the P2SH
address starts with `2` as expected on regtest/testnet.

## Explanation

P2SH validation happens in two independent layers. The outer scriptPubKey only checks
that the spender supplies a redeemScript whose HASH160 matches the committed
`scriptHash` — it is a pure hash-preimage check and says nothing about what the
redeemScript actually requires. Matching the hash proves you know *some* script with that
hash, but it does not by itself authorize spending. The inner check happens after the
redeemScript is revealed and executed: `2 <pub1> <pub2> <pub3> 3 OP_CHECKMULTISIG` then
demands two valid signatures, from two of the three specified private keys, over the
spending transaction. So a spender needs both pieces: the exact redeemScript bytes (to
satisfy the outer hash check) and at least two matching signatures (to satisfy the inner
multisig check). Either one alone is insufficient — knowing the redeemScript without the
signing keys does not let you spend, and having signing keys without the correct
redeemScript bytes does not let you satisfy the outer commitment.
