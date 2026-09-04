# Lab 04 — Native P2WPKH

## Commands used

```bash
cargo test --test lab_04 -- --nocapture
```

## Terminal output

```
running 4 tests
test leaves_scriptsig_empty_and_uses_witness ... ok
test builds_a_version_zero_witness_lock ... ok
test derives_a_native_regtest_address ... ok
test reports_a_twenty_byte_program ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All four public tests in `tests/lab_04.rs` pass against `src/labs/lab04_p2wpkh.rs`:
`derive_p2wpkh_address` matches `Address::p2wpkh(&compressed, regtest)` and starts with `bcrt1q`;
`build_p2wpkh_script_pubkey` matches the expected scriptPubKey and starts with `0014` (witness
version 0 push + 20-byte program length); `witness_program` reports `version: 0`,
`program_length: 20`, and a 40-hex-character program; `native_spend_template` leaves
`script_sig_hex` empty and places `[signature, pubkey]` in `witness_items`.

## Explanation

**Versus P2PKH:** P2PKH's scriptPubKey is a full *script* (`OP_DUP OP_HASH160 <hash>
OP_EQUALVERIFY OP_CHECKSIG`) that a legacy interpreter runs, and the unlocking data (signature +
pubkey) lives in ScriptSig, mixed with the rest of the transaction data that legacy signature
hashing covers. P2WPKH's scriptPubKey instead is just a *witness program* — `0014<20-byte-hash>`,
a version byte plus a fixed-length hash push, not executable opcodes in the legacy sense
(`witness_program` in this lab reports exactly that structure: version 0, a 20-byte program).
The actual `OP_DUP OP_HASH160 ... OP_CHECKSIG`-equivalent logic is implied by the witness version
and run by SegWit-aware nodes against data that now lives in the *witness*, not ScriptSig — which
is why `native_spend_template` leaves `script_sig_hex` empty and puts the signature/pubkey in
`witness_items` instead. This also fixes transaction malleability, since the witness is excluded
from the legacy txid.

**Versus P2SH-wrapped SegWit:** wrapped SegWit (P2SH-P2WPKH) *also* puts data in the witness at
spend time, but its scriptPubKey is a normal P2SH `OP_HASH160 <scriptHash> OP_EQUAL`, and ScriptSig
is not empty — it must contain a push of the redeemScript (`0014<hash>`) so legacy/non-SegWit nodes
still see a superficially valid-looking spend. Native P2WPKH skips that indirection entirely: the
witness program *is* the scriptPubKey directly, and ScriptSig is always empty. That's simpler and
slightly smaller on-chain, at the cost of only being spendable to/recognized by wallets that
understand Bech32/native SegWit addresses (see Lab 05's compatibility discussion).
