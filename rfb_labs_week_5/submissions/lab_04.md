# Lab 04 — Native P2WPKH

## Commands used

```shell
test@pop-os:~/Desktop/rust/rust-for-bitcoin-2.0/rfb_labs_week_5$ cargo test --test lab_04
```

## Terminal output

```terminaloutput
running 4 tests
test leaves_scriptsig_empty_and_uses_witness ... ok
test builds_a_version_zero_witness_lock ... ok
test derives_a_native_regtest_address ... ok
test reports_a_twenty_byte_program ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The derived address on regtest begins with `bcrt1q`. The scriptPubKey built by
`build_p2wpkh_script_pubkey` is a version-0, 20-byte witness program
`OP_0 <20-byte key hash>` (hex prefix `0014`). `witness_program` reports
`version: 0`, `program_length: 20` (40 hex chars). `native_spend_template` returns
an empty `script_sig_hex` and places `<sig> <pubkey>` in `witness_items`.

## Evidence references

```
Code: src/labs/lab04_p2wpkh.rs
Test: tests/lab_04.rs
```

## Explanation

Native P2WPKH (SegWit v0) moves the unlocking data out of the ScriptSig and into a
separate **witness** field, leaving the ScriptSig empty.

In legacy P2PKH the signature and public key are pushed directly into the ScriptSig
(`<sig> <pubkey>`). In native P2WPKH the output's scriptPubKey is simply
`OP_0 <20-byte key hash>` — a witness program that records only the hash of the
public key, exactly as P2PKH records a pubkey hash. The signature and public key are
instead supplied in the witness (`witness_items`), and the ScriptSig (`script_sig_hex`)
is empty.

This differs from P2PKH, where the unlocking data lives in ScriptSig and there is no
witness at all. It also differs from P2SH-wrapped SegWit (P2SH-P2WPKH), where the
20-byte witness program is itself embedded inside a P2SH scriptPubKey
(`OP_HASH160 <scriptHash> OP_EQUAL`), so the outer address still looks like a `3...`
address. Native P2WPKH is "native" because it encodes the version-0 witness program
directly in the scriptPubKey, producing a `bcrt1q...`/`bc1q...` address with no P2SH
wrapper.
