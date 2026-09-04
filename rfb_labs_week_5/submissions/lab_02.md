# Lab 02 — Legacy P2PKH

## Commands used

```
cargo test --test lab_02 -- --nocapture
```

Ad-hoc check against the disposable public key derived from secp256k1 secret key
`[2u8; 32]`:

```rust
let pk2 = disposable_public_key(2);
lab02_p2pkh::derive_p2pkh_address(&pk2.to_string(), Network::Bitcoin)
lab02_p2pkh::build_p2pkh_script_pubkey(&pk2.to_string())
lab02_p2pkh::committed_pubkey_hash(&pk2.to_string())
```

## Terminal output

```
running 4 tests
test puts_unlocking_data_in_scriptsig ... ok
test builds_the_standard_p2pkh_lock ... ok
test commits_to_hash160_of_the_public_key ... ok
test derives_the_expected_p2pkh_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```
p2pkh address = 1NVYv5jmr9JRF3usPZJQmJFJhbQhrPESTP
script_pubkey = 76a914ebc0ee0b2ab9e8277a600c251475e22a3241a1c188ac
pubkey_hash = ebc0ee0b2ab9e8277a600c251475e22a3241a1c1
```

`p2pkh_spend_template("30440220deadbeef01", <pubkey>)` returns
`script_sig_items = ["30440220deadbeef01", <pubkey hex>]` and an empty `witness_items`,
matching the assertion in `puts_unlocking_data_in_scriptsig`.

## Evidence references

- `cargo test --test lab_02` output above.
- Source: `src/labs/lab02_p2pkh.rs`.
- Test suite: `tests/lab_02.rs`.

## Explanation

P2PKH separates *identity* from *authorization*. The scriptPubKey
(`OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG`) only commits to the
HASH160 of a public key, an identity claim: whoever can produce a key with that hash
and a valid signature for it may spend the output. It does not itself prove
authorization. Authorization is supplied at spend time in the ScriptSig, which places
the raw signature and the actual public key in the unlocking script. The `OP_DUP
OP_HASH160 ... OP_EQUALVERIFY` half confirms the supplied public key matches the
committed identity; the trailing `OP_CHECKSIG` then confirms the supplied signature
was produced by the private key behind that public key over this transaction. Identity
verification and signature verification are two distinct checks chained together, not
one step.
