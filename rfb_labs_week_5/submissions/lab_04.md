# Lab 04 — Native P2WPKH

## Commands used

```bash
cargo test --test lab_04
cargo fmt --check
bash grader/grade.sh
```

## Terminal output

```text
running 4 tests
test leaves_scriptsig_empty_and_uses_witness ... ok
test reports_a_twenty_byte_program ... ok
test builds_a_version_zero_witness_lock ... ok
test derives_a_native_regtest_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Unit tests in `tests/lab_04.rs` verifying native P2WPKH Bech32 address generation (`bcrt1q...`), construction of version-0 20-byte witness programs (`0014...`), empty `script_sig_hex`, and witness stack item placement (`[signature, public_key]`).

## Explanation

Native P2WPKH replaces legacy scriptSig unlocking with a version 0 SegWit witness program (`OP_0 <20-byte-pubkey-hash>`). When spending a native SegWit UTXO, the input's `ScriptSig` field is left completely empty (`0x00` length). Instead, the signature and public key are placed inside the dedicated transaction Witness field. Moving witness data out of `ScriptSig` eliminates third-party transaction malleability (because signatures are no longer part of the legacy TXID hash calculation) and significantly lowers transaction fees due to BIP141 witness weight discount rules.
