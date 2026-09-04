# Lab 02 — Legacy P2PKH construction

## Commands used

```
cargo test --test lab_02 -- --nocapture
cargo fmt --check
```

Implementation: `src/labs/lab02_p2pkh.rs` — `derive_p2pkh_address`,
`build_p2pkh_script_pubkey`, `committed_pubkey_hash`, `p2pkh_spend_template`.

## Terminal output

```
running 4 tests
test commits_to_hash160_of_the_public_key ... ok
test puts_unlocking_data_in_scriptsig ... ok
test builds_the_standard_p2pkh_lock ... ok
test derives_the_expected_p2pkh_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Observed behaviour for the test key (`SecretKey::from_slice(&[2u8; 32])`):
- `committed_pubkey_hash` equals `PublicKey::pubkey_hash()` (HASH160 = RIPEMD160(SHA256(pubkey))).
- `build_p2pkh_script_pubkey` equals `Address::p2pkh(pk, Bitcoin).script_pubkey()` and begins
  `76a914` (`OP_DUP OP_HASH160 <push 20>`) and ends `88ac` (`OP_EQUALVERIFY OP_CHECKSIG`).
- `p2pkh_spend_template` returns `script_sig_items = [<sig>, <pubkey>]` and an empty witness.

## Evidence references

- `src/labs/lab02_p2pkh.rs` — implementation, including hex/key validation before templating.
- `tests/lab_02.rs` — assertions against `rust-bitcoin`'s own `Address::p2pkh` output.
- Terminal block above copied from the `cargo test --test lab_02` run.

## Explanation

P2PKH locks a coin to `OP_DUP OP_HASH160 <h> OP_EQUALVERIFY OP_CHECKSIG`, where
`<h>` is HASH160 of the payee's public key. The output commits only to the *hash*,
so the spender reveals the full public key at spend time. To unlock, the input's
ScriptSig pushes `<signature> <pubkey>`; the script then duplicates the pubkey,
hashes it, checks it equals `<h>` (`OP_EQUALVERIFY`), and finally verifies the
signature over the transaction with `OP_CHECKSIG`. Because this is a pre-SegWit
output, all unlocking data sits in the ScriptSig and the witness is empty; the
signature also covers the ScriptSig structure, which is the malleability weakness
SegWit later fixed. The address is just Base58Check(version=0x00, payload=`<h>`),
so deriving it never needs the private key.
