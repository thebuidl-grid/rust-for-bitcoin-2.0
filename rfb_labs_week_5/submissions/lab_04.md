# Lab 04 — Native P2WPKH

**Author:** [Christopher Dominic Eze](https://github.com/Christopherdominic)

## Commands used

```bash
cargo test --test lab_04
cargo run --example evidence   # scratch script, deleted after copying the output below
```

## Terminal output

```
running 4 tests
test leaves_scriptsig_empty_and_uses_witness ... ok
test builds_a_version_zero_witness_lock ... ok
test derives_a_native_regtest_address ... ok
test reports_a_twenty_byte_program ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Using the disposable test key from `[4u8; 32]`:

```
public key:    03462779ad4aad39514614751a71085f2f10e1c7a593e4e030efb5b8721ce55b0b
P2WPKH address: bcrt1q3zxmh4ue370cp48c9d8eeek43qhnzzhvquj2zm
witness version: 0, program: 888dbbd7998f9f80d4f82b4f9ce6d5882f310aec
scriptSig: "" (empty), witness: ["30440220cafebabe01", "03462779ad4aad39514614751a71085f2f10e1c7a593e4e030efb5b8721ce55b0b"]
```

## Evidence references

- `src/labs/lab04_p2wpkh.rs` — `derive_p2wpkh_address`, `build_p2wpkh_script_pubkey`,
  `witness_program`, `native_spend_template`.
- `tests/lab_04.rs::leaves_scriptsig_empty_and_uses_witness` — asserts
  `script_sig_hex.is_empty()` and `witness_items == [sig, pubkey]`, matching above.
- `tests/lab_04.rs::builds_a_version_zero_witness_lock` — asserts the scriptPubKey
  starts with `0014` (`OP_0`, push 20 bytes), the version-0 P2WPKH witness program.

## Explanation

Native P2WPKH puts absolutely nothing in the legacy ScriptSig field — it's an empty
byte string, full stop. Everything needed to unlock the output (the signature and the
public key) goes in the witness, which is a separate, segregated part of the
transaction that was introduced by BIP141. That's the "segregated" in Segregated
Witness: the unlocking data is pulled out of the part of the transaction that's
hashed to produce the txid, which is what fixes transaction malleability — you can't
change someone's txid by tweaking their signature encoding anymore, because the
signature isn't part of what the txid commits to.

Compared to plain P2PKH: P2PKH's scriptPubKey is
`OP_DUP OP_HASH160 <hash> OP_EQUALVERIFY OP_CHECKSIG`, executed against data sitting
in the ScriptSig. P2WPKH's scriptPubKey is just `OP_0 <20-byte-hash>` — a witness
version byte and a program, no opcodes to actually run against the ScriptSig, because
the ScriptSig is empty by consensus rule for a native SegWit output. Verification
logic for a witness program is special-cased by the node rather than being general
script execution against the ScriptSig; effectively the same `DUP/HASH160/EQUALVERIFY/CHECKSIG`
logic runs, but against the witness stack instead.

Compared to P2SH-wrapped SegWit (P2SH-P2WPKH, BIP49): that format is a compromise for
wallets/exchanges that could only send to `3...` addresses. The witness program still
lives in the witness and the ScriptSig is still nearly empty at redemption — but it's
not *quite* empty, because the ScriptSig has to contain a single push of the
redeemScript (`OP_0 <hash>`) so the outer P2SH hash check has something to verify. So
you get P2SH's Base58Check compatibility layered on top of SegWit's actual witness
mechanics, at the cost of a few extra scriptSig bytes and an extra hashing step.
Native P2WPKH skips that wrapper entirely: no outer P2SH layer, no non-empty
ScriptSig, just the witness program directly in the scriptPubKey — smaller, and it's
why native SegWit gets the full fee benefit while wrapped SegWit gives some of it back
to the P2SH ScriptSig push.
