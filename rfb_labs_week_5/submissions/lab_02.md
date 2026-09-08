# Lab 02 — Legacy P2PKH

## Commands used

```bash
cargo test --test lab_02
cargo fmt --check
bash grader/grade.sh
```

## Terminal output

```text
running 4 tests
test commits_to_hash160_of_the_public_key ... ok
test puts_unlocking_data_in_scriptsig ... ok
test builds_the_standard_p2pkh_lock ... ok
test derives_the_expected_p2pkh_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Automated unit test execution in `tests/lab_02.rs` verifying P2PKH address generation, construction of standard locking scripts (`OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG`), HASH160 public key hash commitments, and placement of signature and public key in ScriptSig with empty witness.

## Explanation

P2PKH locking commits to the HASH160 (RIPEMD160(SHA256)) hash of a compressed public key, which keeps UTXO scriptPubKeys compact (25 bytes) and conceals the public key until the coin is spent. Unlocking requires providing both an ECDSA signature and the revealed public key in `ScriptSig`. During execution, the Script interpreter executes `OP_DUP` to duplicate the public key, `OP_HASH160` to hash it, and `OP_EQUALVERIFY` to confirm that the hash matches the commitment in scriptPubKey. Finally, `OP_CHECKSIG` verifies that the provided signature is valid for that public key and transaction digest.
