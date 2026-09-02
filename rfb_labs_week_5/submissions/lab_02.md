# Lab 02 — Legacy P2PKH construction

## Commands used

```bash
cargo test --test lab_02 -- --nocapture
cargo fmt --check
cargo clippy --all-targets
```

Implementation lives in `src/labs/lab02_p2pkh.rs`: `derive_p2pkh_address`,
`build_p2pkh_script_pubkey`, `committed_pubkey_hash`, and
`p2pkh_spend_template`.

## Terminal output

```
running 4 tests
test derives_the_expected_p2pkh_address ... ok
test commits_to_hash160_of_the_public_key ... ok
test puts_unlocking_data_in_scriptsig ... ok
test builds_the_standard_p2pkh_lock ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

- `src/labs/lab02_p2pkh.rs` — HASH160 commitment, `OP_DUP OP_HASH160 <hash>
  OP_EQUALVERIFY OP_CHECKSIG` script construction, and the ScriptSig template.
- `tests/lab_02.rs` — public suite exercised above, all 4 tests green;
  `puts_unlocking_data_in_scriptsig` checks `script_sig_items` equals
  `[signature, pubkey]` and `witness_items` is empty.
- `bash grader/grade.sh` recorded `02 | 4/4 | 4 | ...` for this lab.

## Explanation

P2PKH separates *who this coin belongs to* from *what unlocks it*. The
scriptPubKey (`OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG`)
only commits to a HASH160 of the public key — that hash is the coin's identity
claim, and `committed_pubkey_hash` shows it is derived purely from the public
key, with no signature involved at all. Knowing the public key that hashes to
that commitment proves identity but proves nothing about authorization: HASH160
is a one-way function, so revealing the public key that matches it does not
require having signed anything.

Spend authorization is the second, independent step performed in ScriptSig at
spend time: `p2pkh_spend_template` places `[signature, public_key]` there, and
the interpreter runs `OP_CHECKSIG`, which cryptographically verifies the
signature against the current transaction using the private key that
corresponds to the revealed public key. Only a party holding that private key
can produce a signature that satisfies `OP_CHECKSIG`. So the locking script
commits to identity (the hash), while the unlocking script proves control
(a valid signature) — matching the hash alone (e.g. by just supplying the
public key) would satisfy `OP_EQUALVERIFY` but leave `OP_CHECKSIG` unsatisfied,
which is exactly why both checks exist in the same script.
