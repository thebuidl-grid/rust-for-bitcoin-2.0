# Lab 04 — Native P2WPKH

## Commands used

```bash
cargo test --test lab_04
cargo fmt --check
```

## Terminal output

```text
$ cargo test --test lab_04
running 4 tests
test leaves_scriptsig_empty_and_uses_witness ... ok
test reports_a_twenty_byte_program ... ok
test builds_a_version_zero_witness_lock ... ok
test derives_a_native_regtest_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo fmt --check` produced no diff.

## Evidence references

- Implementation: `src/labs/lab04_p2wpkh.rs`
- Test suite: `tests/lab_04.rs`
- `derive_p2wpkh_address` produces a `bcrt1q...` address matching
  `Address::p2wpkh(&compressed, Network::Regtest)` exactly.
- `build_p2wpkh_script_pubkey` returns a scriptPubKey starting with `0014` — `OP_0` followed by a
  20-byte push, confirmed by `builds_a_version_zero_witness_lock`.
- `witness_program` reports `version: 0`, `program_length: 20`, and a 40-hex-character program,
  read straight off the address's real `WitnessProgram` via `reports_a_twenty_byte_program`.
- `native_spend_template` leaves `script_sig_hex` empty and places `[signature, pubkey]` in
  `witness_items`, per `leaves_scriptsig_empty_and_uses_witness`.

## Explanation

Native P2WPKH moves the unlocking data out of ScriptSig entirely and into the witness, which is
what makes its ScriptSig empty. In legacy P2PKH, the signature and public key live in ScriptSig,
which is part of the transaction data that gets hashed for the legacy transaction ID (txid) and
counted at full weight. BIP141 (SegWit) instead defines a separate "witness" section that sits
outside the base transaction and is *not* part of the txid computation, and — crucially — is
discounted in the BIP141 weight formula (witness bytes count once instead of four times). Because
P2WPKH's locking script is a version-0, 20-byte witness program (`0 <pubKeyHash>`) rather than the
old `OP_DUP OP_HASH160 ... OP_CHECKSIG` template, a legacy node that doesn't understand witness
data still sees a spendable-looking output, and pre-SegWit relay code that only inspects
ScriptSig sees nothing there at all — because there's genuinely nothing to see: the proof of
ownership has moved to a place that's malleability-resistant (it doesn't affect the txid) and
cheaper (discounted weight), which is the whole point of the P2SH-wrapped vs. native SegWit
transition covered in Lab 05.

