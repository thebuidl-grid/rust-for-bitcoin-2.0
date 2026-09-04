# Lab 04 — Native SegWit P2WPKH

## Commands used

```
cargo test --test lab_04 -- --nocapture
cargo fmt --check
```

Implementation: `src/labs/lab04_p2wpkh.rs` — `derive_p2wpkh_address`,
`build_p2wpkh_script_pubkey`, `witness_program`, `native_spend_template`.

## Terminal output

```
running 4 tests
test builds_a_version_zero_witness_lock ... ok
test derives_a_native_regtest_address ... ok
test leaves_scriptsig_empty_and_uses_witness ... ok
test reports_a_twenty_byte_program ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Observed behaviour for test key `SecretKey::from_slice(&[4u8; 32])`:
- `derive_p2wpkh_address` on regtest returns a `bcrt1q...` string (bech32, HRP `bcrt`).
- `build_p2wpkh_script_pubkey` returns `0014<20-byte-hash>` — `OP_0` then a 20-byte push.
- `witness_program` reports version `0`, program length `20`, 40 hex chars.
- `native_spend_template` returns an empty `script_sig_hex` and `witness_items = [<sig>, <pubkey>]`.

## Evidence references

- `src/labs/lab04_p2wpkh.rs` — requires a compressed key via `CompressedPublicKey::try_from`.
- `tests/lab_04.rs` — checks the `bcrt1q` prefix, the `0014` script, and the empty ScriptSig.
- Terminal block above from `cargo test --test lab_04`.

## Explanation

P2WPKH is the SegWit v0 equivalent of P2PKH. The scriptPubKey is a *witness
program*: a version byte (`OP_0`) followed by a 20-byte push of HASH160(compressed
pubkey). There is no `OP_DUP`/`OP_CHECKSIG` in the output — those steps are implied
by the version-0, 20-byte shape. Because the consensus rules for SegWit move all
unlocking data into a separate `witness` field, the ScriptSig for a native P2WPKH
input is *empty*. The witness stack holds exactly `[<signature>, <pubkey>]`, the
same two items P2PKH put in the ScriptSig, but now outside the part of the
transaction that is hashed into the txid. That is what removes third-party
malleability and lets the witness be discounted in weight. The key must be
compressed (33 bytes); an uncompressed key is invalid for v0 witness programs.
