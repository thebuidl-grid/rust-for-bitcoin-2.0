# Lab 02 — Legacy P2PKH construction

## Commands used

```bash
cargo test --test lab_02 -- --nocapture
```

## Terminal output

```
running 4 tests
test builds_the_standard_p2pkh_lock ... ok
test derives_the_expected_p2pkh_address ... ok
test puts_unlocking_data_in_scriptsig ... ok
test commits_to_hash160_of_the_public_key ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Evidence references

`src/labs/lab02_p2pkh.rs`:

- `derive_p2pkh_address` parses the compressed public key and calls
  `bitcoin::Address::p2pkh(public, network)`.
- `build_p2pkh_script_pubkey` builds `ScriptBuf::new_p2pkh(&public.pubkey_hash())`, which
  encodes exactly `OP_DUP OP_HASH160 <20-byte-hash> OP_EQUALVERIFY OP_CHECKSIG`.
- `committed_pubkey_hash` returns `public.pubkey_hash().to_string()` — the same HASH160
  the scriptPubKey commits to.
- `p2pkh_spend_template` places `[signature_hex, public_key_hex]` in
  `script_sig_items` and leaves `witness_items` empty, matching how a legacy input is
  unlocked (all data lives in ScriptSig; SegWit's witness stack is unused).

## Explanation

The scriptPubKey only commits to identity: HASH160(pubkey). That's public information
by definition — anyone who's ever seen the address knows that hash, and knowing it
proves nothing about who's allowed to spend the coin.

Spend authorization is a different thing entirely: a signature over the specific
spending transaction, produced with the private key. `OP_CHECKSIG` is what ties the
two together at spend time — it re-derives HASH160(pubkey) from the ScriptSig data,
checks it matches what `OP_EQUALVERIFY` already confirmed, then verifies the signature
against the sighash using that same pubkey. The identity check and the authorization
check happen back to back but they're checking different things. That's the whole
trick of P2PKH: the lock is visible to the entire chain, but only the private key
holder can ever produce a signature that passes the second half of the check.
