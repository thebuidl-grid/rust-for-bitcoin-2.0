# Lab 02 — Legacy P2PKH

**Author:** [Christopher Dominic Eze](https://github.com/Christopherdominic)

## Commands used

```bash
cargo test --test lab_02
cargo run --example evidence   # scratch script, deleted after copying the output below
```

## Terminal output

```
running 4 tests
test builds_the_standard_p2pkh_lock ... ok
test puts_unlocking_data_in_scriptsig ... ok
test commits_to_hash160_of_the_public_key ... ok
test derives_the_expected_p2pkh_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Using the disposable test key `SecretKey::from_slice(&[2u8; 32])` from the test file:

```
public key:    024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d0766
P2PKH address: 1NVYv5jmr9JRF3usPZJQmJFJhbQhrPESTP
pubkey hash:   ebc0ee0b2ab9e8277a600c251475e22a3241a1c1
scriptPubKey:  76a914ebc0ee0b2ab9e8277a600c251475e22a3241a1c188ac
```

## Evidence references

- `src/labs/lab02_p2pkh.rs` — `derive_p2pkh_address`, `build_p2pkh_script_pubkey`,
  `committed_pubkey_hash`, `p2pkh_spend_template`.
- `tests/lab_02.rs::puts_unlocking_data_in_scriptsig` — asserts
  `script_sig_items == ["30440220deadbeef01", <pubkey>]` and `witness_items` is
  empty, which matches the values above.
- The scriptPubKey (`76a914` + 20-byte hash + `88ac`) is
  `OP_DUP OP_HASH160 <hash> OP_EQUALVERIFY OP_CHECKSIG`; the hash in the middle is the
  same `ebc0ee0b...` value reported by `committed_pubkey_hash`.

## Explanation

P2PKH locks a coin to a hash of a public key, not to the key itself. What actually
gets committed on-chain in the scriptPubKey is `HASH160(pubkey)` — that's the "key
identity" part. Anyone can look at that hash and know which key is allowed to spend
the output, and it's compact (20 bytes vs. 33/65 for the key), but the hash on its own
proves nothing about who's allowed to move the coin.

Spend authorization is the separate step that happens later, in the ScriptSig, when
someone actually wants to spend. The unlocking script has to supply two things: the
raw public key (so the node can re-hash it and check it matches the committed hash),
and a signature over the spending transaction, produced with the matching private
key. The interpreter runs `OP_DUP OP_HASH160 <hash> OP_EQUALVERIFY` to check the
supplied key matches the commitment, then `OP_CHECKSIG` to check the signature is
valid for that key. Both checks have to pass — matching the hash without a valid
signature gets you nowhere, and a valid signature for the wrong key doesn't match the
hash in the first place.

That split is basically the whole point of P2PKH: the locking script commits to an
identity cheaply and without revealing the actual public key until spend time, while
the unlocking script is where you prove control by revealing the key and signing.
It's also why the spend template puts both `signature_hex` and `public_key_hex` into
`script_sig_items` and leaves `witness_items` empty — this is a pre-SegWit script
type, so all of the unlocking data lives in the legacy ScriptSig field, not in a
witness stack.
