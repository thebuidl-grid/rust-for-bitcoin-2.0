# Lab 02 — Legacy P2PKH

## Commands used

```bash
cargo test --test lab_02
cargo fmt --check
```

## Terminal output

```text
$ cargo test --test lab_02
running 4 tests
test puts_unlocking_data_in_scriptsig ... ok
test builds_the_standard_p2pkh_lock ... ok
test commits_to_hash160_of_the_public_key ... ok
test derives_the_expected_p2pkh_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo fmt --check` produced no diff.

## Evidence references

- Implementation: `src/labs/lab02_p2pkh.rs`
- Test suite: `tests/lab_02.rs`
- `derive_p2pkh_address` matches `Address::p2pkh(public, network).to_string()` exactly
  (`derives_the_expected_p2pkh_address`).
- `build_p2pkh_script_pubkey` produces `ScriptBuf::new_p2pkh(&public.pubkey_hash())`, which the
  test independently reconstructs via `Address::p2pkh(...).script_pubkey()` and finds equal.
- `committed_pubkey_hash` returns `public_key.pubkey_hash().to_string()`, the same HASH160 value
  the scriptPubKey commits to.
- `p2pkh_spend_template` places `["30440220deadbeef01", <pubkey>]` in `script_sig_items` and
  leaves `witness_items` empty, matching `puts_unlocking_data_in_scriptsig`.

## Explanation

P2PKH separates *key identity* from *spend authorization*. The scriptPubKey
(`OP_DUP OP_HASH160 <pubKeyHash> OP_EQUALVERIFY OP_CHECKSIG`) only commits to the HASH160 of a
public key — that's identity: "whoever holds the key that hashes to this value may spend this
output." It does not by itself authorize anything. Authorization happens later, at spend time, in
the ScriptSig: the spender must supply both the actual public key (proving it hashes to the
committed value via `OP_EQUALVERIFY`) and a valid ECDSA signature over the spending transaction
(checked by `OP_CHECKSIG`). Knowing the public key alone proves nothing — anyone can compute a
HASH160 or read a public key off the chain — only a signature produced by the corresponding
private key proves the right to spend. That's why the lock only stores a hash (identity) while
the unlock must contain a live signature (authorization): a leaked public key threatens privacy,
not funds, but a leaked private key (or a forged signature) threatens funds directly.

