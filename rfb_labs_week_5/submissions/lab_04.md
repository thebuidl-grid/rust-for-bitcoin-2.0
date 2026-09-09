# Lab 04 — Native P2WPKH

## Commands used

```bash
cargo test --test lab_04
bash grader/grade.sh
```

## Terminal output

running 4 tests

test builds_a_version_zero_witness_lock ... ok

test leaves_scriptsig_empty_and_uses_witness ... ok

test derives_a_native_regtest_address ... ok

test reports_a_twenty_byte_program ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

## Evidence references


All four public tests in `tests/lab_04.rs` pass, covering:
- Deriving a native `bcrt1q...` regtest address from a compressed public key
  (`derives_a_native_regtest_address`)
- Building the version-0 `0014<20-byte-hash>` scriptPubKey
  (`builds_a_version_zero_witness_lock`)
- Reporting a witness version of 0 and a 20-byte (40 hex character) program
  (`reports_a_twenty_byte_program`)
- Confirming ScriptSig is empty and the signature/public key are placed in
  the witness instead (`leaves_scriptsig_empty_and_uses_witness`)

## Explanation

Native P2WPKH has an empty ScriptSig because all of its unlocking data —
the signature and the public key — moved to a separate structure entirely:
the witness. This is the defining change BIP141 (Segregated Witness)
introduced.

In legacy P2PKH (Lab 02), the signature and public key both live inside
ScriptSig, which is part of the transaction data that gets hashed when
computing the transaction's txid. In P2SH-wrapped SegWit, ScriptSig still
isn't empty either — it holds a small push of the redeemScript needed to
satisfy the outer P2SH hash check, even though the actual signature/pubkey
have moved to the witness. Native P2WPKH is the only one of the three where
ScriptSig is completely empty, because there's no outer P2SH commitment to
satisfy and no legacy ScriptSig convention to preserve — the scriptPubKey
itself is just the version-0 witness program (`0014<hash>`), and everything
needed to spend it lives in the witness field.

This separation is what "segregated" in "Segregated Witness" refers to:
witness data is no longer part of the data used to compute a transaction's
txid, which fixes transaction malleability (since a witness can no longer
be altered to change the txid without invalidating the transaction) and
lets witness data be discounted in weight/fee calculations, since nodes
that don't need to verify signatures (like SPV clients) can ignore it
entirely.

