# Lab 04 — Native P2WPKH

## Commands used

```bash
cargo test --test lab_04 -- --nocapture
```

## Terminal output

```
running 4 tests
test leaves_scriptsig_empty_and_uses_witness ... ok
test derives_a_native_regtest_address ... ok
test builds_a_version_zero_witness_lock ... ok
test reports_a_twenty_byte_program ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Evidence references

`src/labs/lab04_p2wpkh.rs`:

- `derive_p2wpkh_address` requires a `CompressedPublicKey` (via
  `CompressedPublicKey::try_from(public)`, which rejects uncompressed keys — SegWit
  mandates compressed pubkeys) and calls `Address::p2wpkh(&compressed, network)`; the
  regtest test asserts the result starts with `bcrt1q`.
- `build_p2wpkh_script_pubkey` returns `ScriptBuf::new_p2wpkh(&compressed.wpubkey_hash())`,
  confirmed by the test to start with `0014` — witness version 0 (`OP_0`) followed by a
  20-byte push.
- `witness_program` reports `version: 0`, a 20-byte `program_hex`
  (`compressed.wpubkey_hash()`), matching BIP141's P2WPKH witness program.
- `native_spend_template` leaves `script_sig_hex` empty and puts
  `[signature_hex, public_key_hex]` in `witness_items` instead.

## Explanation

Same idea underneath — lock to a hash of a public key — but three different places
for the unlocking data to live, and that's what actually costs money.

P2PKH: signature and pubkey both go in ScriptSig, counted at full weight, no witness
involved at all. P2SH-wrapped SegWit keeps the old scriptPubKey shape on the outside
(`OP_HASH160 <scriptHash> OP_EQUAL`) so legacy senders can still build an output to
it, but the redeemScript hidden inside is a P2WPKH program — so ScriptSig only
carries that small redeemScript push, and the actual signature + pubkey move into the
witness. Native P2WPKH skips the wrapper entirely: the scriptPubKey *is* the witness
program (`0 <20-byte-hash>`), ScriptSig is empty, everything unlocking-related sits in
witness data.

Why it matters: BIP141 weights witness bytes at 1/4 of ScriptSig bytes. P2PKH gets no
discount. P2SH-wrapped SegWit gets a partial discount (small legacy wrapper cost,
discounted witness). Native P2WPKH gets the full discount and skips the wrapper cost
too — which is exactly the vbyte gap this lab's tests are built to demonstrate.
