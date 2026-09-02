# Lab 04 — Native P2WPKH

## Commands used

```bash
cargo test --test lab_04 -- --nocapture
cargo fmt --check
cargo clippy --all-targets
```

Implementation lives in `src/labs/lab04_p2wpkh.rs`: `derive_p2wpkh_address`,
`build_p2wpkh_script_pubkey`, `witness_program`, and `native_spend_template`.

## Terminal output

```
running 4 tests
test builds_a_version_zero_witness_lock ... ok
test derives_a_native_regtest_address ... ok
test reports_a_twenty_byte_program ... ok
test leaves_scriptsig_empty_and_uses_witness ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

- `src/labs/lab04_p2wpkh.rs` — the version-0, 20-byte witness program and the
  `bcrt1q...` address it encodes.
- `tests/lab_04.rs` — `builds_a_version_zero_witness_lock` checks the
  scriptPubKey hex starts with `0014` (witness v0 push of a 20-byte program);
  `reports_a_twenty_byte_program` checks `program_length == 20`;
  `leaves_scriptsig_empty_and_uses_witness` checks `script_sig_hex` is empty
  and the signature/pubkey live in `witness_items` instead.
- `bash grader/grade.sh` recorded `04 | 4/4 | 4 | ...` for this lab.

## Explanation

P2WPKH keeps the same commitment as P2PKH — a HASH160 of the compressed
public key — but relocates both the commitment and the unlocking data outside
the legacy script system entirely. In P2PKH the hash sits inside a scriptPubKey
program (`OP_DUP OP_HASH160 <hash> OP_EQUALVERIFY OP_CHECKSIG`) that the
legacy interpreter executes, and the signature/pubkey pair sits in ScriptSig,
which counts toward the pre-SegWit, non-discounted part of transaction size.
P2WPKH instead stores the hash as a bare witness program — `witness_program`
reports it directly as `(version = 0, program = 20-byte hash)`, with no
opcodes at all in the scriptPubKey beyond the version push — and moves the
signature/public key into the witness field, which `native_spend_template`
shows explicitly: `script_sig_hex` is empty while `witness_items` holds
`[signature, pubkey]`. Because it is witness data, that content is discounted
under BIP141 weight accounting, which is what produces the fee savings
compared to P2PKH.

It also differs from P2SH-wrapped SegWit (BIP49), which is the migration
format for wallets that cannot yet parse `bc1...` addresses: there, the
scriptPubKey is a legacy `OP_HASH160 <scriptHash> OP_EQUAL` P2SH lock, and the
ScriptSig — not empty — contains a single push of the redeemScript
(`0 <pubKeyHash>`), which the node unwraps before checking the witness. Native
P2WPKH has no P2SH wrapper: the witness program is the scriptPubKey itself,
ScriptSig is always empty, and the address is Bech32-encoded (`bcrt1q...`)
rather than Base58Check (`2...`), so only a wallet that understands SegWit
witness programs directly can recognize and pay it.
