# Lab 02 — Legacy P2PKH construction

## Commands used

```bash
cargo test --test lab_02 -- --nocapture
```

## Terminal output

```
running 4 tests
test puts_unlocking_data_in_scriptsig ... ok
test builds_the_standard_p2pkh_lock ... ok
test derives_the_expected_p2pkh_address ... ok
test commits_to_hash160_of_the_public_key ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All four public tests in `tests/lab_02.rs` pass against `src/labs/lab02_p2pkh.rs`:
`derive_p2pkh_address` matches `Address::p2pkh(public, network)` exactly;
`build_p2pkh_script_pubkey` matches `ScriptBuf::new_p2pkh(&public.pubkey_hash())`, i.e.
`OP_DUP OP_HASH160 <hash> OP_EQUALVERIFY OP_CHECKSIG`; `committed_pubkey_hash` matches
`public.pubkey_hash().to_string()`; `p2pkh_spend_template` places `[signature, pubkey]` in
`script_sig_items` and leaves `witness_items` empty, matching legacy (pre-SegWit) spending rules.

## Explanation

**Key identity** is what the locking script commits to: `committed_pubkey_hash` is the HASH160 of
the public key baked into the scriptPubKey (`OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY
OP_CHECKSIG`). Anyone can see this hash on-chain; it says "funds sent here belong to whoever holds
the private key matching this public key." It proves *who the funds are for*, not that anyone has
actually authorized moving them.

**Spend authorization** is a separate, later step: it's the ECDSA signature over the specific
transaction, supplied in ScriptSig only at spend time (`p2pkh_spend_template`'s
`script_sig_items`). The node re-derives the pubkey hash from the supplied public key
(`OP_DUP OP_HASH160 ... OP_EQUALVERIFY`) to confirm it matches the committed identity, *then*
runs `OP_CHECKSIG` to confirm the signature is valid for that public key over this exact
transaction. Identity alone (just knowing/publishing the public key) never lets you spend —
you need the private key to produce a signature that satisfies `OP_CHECKSIG`. That's why
`witness_items` stays empty here: legacy P2PKH keeps both the identity check and the
authorization proof together in ScriptSig, unlike SegWit which separates them into a witness.
