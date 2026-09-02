# Lab 04 — Native P2WPKH

## Commands used
```
cargo fmt
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --test lab_04
```
## Terminal output
```
running 4 tests
test reports_a_twenty_byte_program ... ok
test leaves_scriptsig_empty_and_uses_witness ... ok
test derives_a_native_regtest_address ... ok
test builds_a_version_zero_witness_lock ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
## Evidence references

The test output above

The tests verify that:

- a native Regtest P2WPKH address is derived correctly
- the witness version is 0
- the witness program is 20 bytes
- the P2WPKH scriptPubKey is constructed correctly
- ScriptSig remains empty
- the signature and public key are placed in the witness

No Bitcoin Core, Polar, or live funds are required for this lab.

## Explanation
Native P2WPKH has an empty ScriptSig because the spending data is placed in the
transaction's witness instead of the traditional ScriptSig.

The P2WPKH output contains a version-0 witness program that commits to the
20-byte HASH160 of the recipient's compressed public key. When the output is
spent, the spender provides the signature and public key in the witness. Bitcoin
uses those witness items to verify that the public key matches the committed hash
and that the signature authorizes the transaction.

Because the witness is specifically designed to carry this unlocking data,
there is nothing that needs to be placed in ScriptSig for a native P2WPKH spend.
Therefore, the ScriptSig is empty.

This is different from legacy P2PKH, where the signature and public key are placed
directly in ScriptSig. In native P2WPKH, the same kind of spending proof is still
required, but it lives in the witness instead.