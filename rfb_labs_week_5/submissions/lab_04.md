# Lab 04 — Native P2WPKH

## Commands used

```
cargo test --test lab_04 -- --nocapture
```

## Terminal output

```
running 4 tests
test builds_a_version_zero_witness_lock ... ok
test leaves_scriptsig_empty_and_uses_witness ... ok
test derives_a_native_regtest_address ... ok
test reports_a_twenty_byte_program ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Using the disposable test secret key `[0x04; 32]`:

```
p2wpkh address (regtest): bcrt1q3zxmh4ue370cp48c9d8eeek43qhnzzhvquj2zm
witness program:          WitnessProgramReport { version: 0, program_hex: "888dbbd7998f9f80d4f82b4f9ce6d5882f310aec", program_length: 20 }
```

`native_spend_template("30440220cafebabe01", <pubkey>)` produced an empty
`script_sig_hex` and placed the signature and public key in `witness_items` instead.

## Explanation

Native P2WPKH moves the unlocking data out of ScriptSig and into the witness field
entirely, so ScriptSig is empty by design — the version-0, 20-byte witness program in the
scriptPubKey (`OP_0 <20-byte-pubkey-hash>`) is what a SegWit-aware node matches against
the witness stack, not against any legacy script execution. This differs from classic
P2PKH, where the signature and public key both live in ScriptSig and are hashed into the
legacy transaction ID (making the txid mutable by third parties who tweak signature
encoding). It also differs from P2SH-wrapped SegWit, which keeps a 1-byte-push ScriptSig
(containing just the witness program as the "redeemScript") purely so older, non-SegWit
software still sees a spendable-looking P2SH output — native P2WPKH drops even that
compatibility shim because it assumes the receiving/relaying software already understands
SegWit. Both SegWit variants get the witness discount and quarantine spending data outside
the legacy-serialized transaction, fixing third-party malleability; only the wrapped
variant pays the extra cost of the redundant P2SH scriptSig byte.
