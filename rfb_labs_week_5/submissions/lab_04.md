# Lab 04 — Native P2WPKH

## Commands used

```
cargo test --test lab_04 -- --nocapture
```

Ad-hoc check using the disposable public key derived from secp256k1 secret key
`[4u8; 32]`:

```rust
let pk4 = disposable_public_key(4);
lab04_p2wpkh::derive_p2wpkh_address(&pk4.to_string(), Network::Regtest)
lab04_p2wpkh::witness_program(&pk4.to_string())
```

## Terminal output

```
running 4 tests
test leaves_scriptsig_empty_and_uses_witness ... ok
test builds_a_version_zero_witness_lock ... ok
test reports_a_twenty_byte_program ... ok
test derives_a_native_regtest_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```
p2wpkh address = bcrt1q3zxmh4ue370cp48c9d8eeek43qhnzzhvquj2zm
witness version = 0, program = 888dbbd7998f9f80d4f82b4f9ce6d5882f310aec (len 20)
```

`native_spend_template("30440220cafebabe01", <pubkey>)` returns an empty
`script_sig_hex` and `witness_items = ["30440220cafebabe01", <pubkey hex>]`.

## Evidence references

- `cargo test --test lab_04` output above.
- Source: `src/labs/lab04_p2wpkh.rs`.
- Test suite: `tests/lab_04.rs`.

## Explanation

Native P2WPKH moves the unlocking data out of ScriptSig entirely and into the witness
field introduced by BIP141. The scriptPubKey is just `0 <20-byte-pubkey-hash>`, a
version-0 witness program, and the ScriptSig for a native SegWit input is always
empty by consensus rule; the signature and public key travel in the transaction's
witness stack instead. This differs from P2PKH, where the signature and public key
sit in ScriptSig and count fully toward the transaction's legacy byte size. It also
differs from P2SH-wrapped SegWit, where the scriptPubKey still looks like ordinary
P2SH (`OP_HASH160 <hash> OP_EQUAL`) and the redeemScript containing the witness
program is placed in ScriptSig so that legacy nodes and wallets can still relay it;
native P2WPKH has no such wrapper and is only spendable by SegWit-aware software. The
empty ScriptSig is precisely what lets witness data be discounted under BIP141's
weight formula, since witness bytes are counted separately from the base
transaction.
