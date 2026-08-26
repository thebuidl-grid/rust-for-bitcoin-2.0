# Lab 04 — Native P2WPKH

## Commands used

```bash
cargo test --test lab_04 -- --nocapture
cargo run --example labs_demo
```

## Terminal output

```text
$ cargo test --test lab_04 -- --nocapture
running 4 tests
test leaves_scriptsig_empty_and_uses_witness ... ok
test builds_a_version_zero_witness_lock ... ok
test reports_a_twenty_byte_program ... ok
test derives_a_native_regtest_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
$ cargo run --example labs_demo   (Lab 04 section, disposable key [4u8; 32])
derive_p2wpkh_address = Ok("bcrt1q3zxmh4ue370cp48c9d8eeek43qhnzzhvquj2zm")
build_p2wpkh_script_pubkey = Ok("0014888dbbd7998f9f80d4f82b4f9ce6d5882f310aec")
witness_program = Ok(WitnessProgramReport { version: 0,
    program_hex: "888dbbd7998f9f80d4f82b4f9ce6d5882f310aec", program_length: 20 })
native_spend_template = Ok(NativeSegwitSpend { script_sig_hex: "",
    witness_items: ["30440220cafebabe01",
                     "03462779ad4aad39514614751a71085f2f10e1c7a593e4e030efb5b8721ce55b0b"] })
```

## Evidence references

- Implementation: [`src/labs/lab04_p2wpkh.rs`](../src/labs/lab04_p2wpkh.rs)
- Public test suite: [`tests/lab_04.rs`](../tests/lab_04.rs) — 4/4 passing, logged in
  [`grading/logs/lab_04.log`](../grading/logs/lab_04.log).
- `build_p2wpkh_script_pubkey` output `0014<20 bytes>` — the version-0 opcode `OP_0`
  (`00`) followed by a 20-byte push (`14`), i.e. the witness program.
- The address `bcrt1q...` starts with the `bcrt1q` regtest native-SegWit v0 prefix.
- `native_spend_template.script_sig_hex` is empty; the signature and public key are
  present only in `witness_items`.

## Explanation

Native P2WPKH's scriptPubKey, `OP_0 <20-byte-pubkey-hash>` (a "version-0 witness
program"), is structurally different from both formats studied earlier:

- **Versus P2PKH**: P2PKH's locking script is a full four-opcode script
  (`OP_DUP OP_HASH160 ... OP_EQUALVERIFY OP_CHECKSIG`) that the spender's ScriptSig
  feeds directly. P2WPKH's on-chain scriptPubKey is just `OP_0` plus the raw
  20-byte hash — there is no `OP_CHECKSIG` in it at all. The equivalent of the P2PKH
  script logic is executed *implicitly* by consensus rules that interpret a version-0,
  20-byte program as "P2WPKH" and reconstruct the P2PKH-equivalent script internally
  from the witness data, rather than storing that script on-chain.
- **Versus P2SH-wrapped SegWit**: a P2SH-wrapped P2WPKH output still has an ordinary
  `OP_HASH160 <hash> OP_EQUAL` scriptPubKey on-chain (so legacy-only nodes/wallets see
  a normal-looking P2SH output), and its ScriptSig is *not* empty — it must push the
  redeemScript (`0 <20-byte-hash>`) so the outer hash check passes, with the signature
  and pubkey still living in the witness. Native P2WPKH skips the wrapper entirely:
  there is no redeemScript to push, so ScriptSig is completely empty
  (`native_spend_template.script_sig_hex == ""`), and the signature/pubkey pair goes
  straight into the witness (`witness_items`), which is exactly what
  `leaves_scriptsig_empty_and_uses_witness` checks.

Moving the unlocking data out of ScriptSig and into the witness is also what BIP141
uses to give SegWit transactions a smaller weight (Lab 06): witness bytes are counted
once instead of the four times a ScriptSig byte would be, so the same signature and
public key cost less block space here than they would under legacy P2PKH.
