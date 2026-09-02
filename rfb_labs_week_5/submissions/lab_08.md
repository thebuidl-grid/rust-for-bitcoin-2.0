# Lab 08 — BIP32 extended keys

## Commands used

`cargo test --test lab_08`

## Terminal output

```bash
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running tests/lab_08.rs (target/debug/deps/lab_08-d5b9a76bd06010a2)

running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
``` 


## Evidence references

Code: src/labs/lab08_bip32.rs  
Test: tests/lab_08.rs

## Explanation

### xpriv, xpub, chain code, and hardened versus normal derivation.

An xpriv is an extended private key. It contains a private key plus the information needed to derive child private keys: the chain code, depth, parent fingerprint, child index, and network/version information.

An xpub is an extended public key. It contains a public key and the same kind of derivation metadata, including a chain code. It can derive public child keys and addresses, but it cannot derive private keys or sign transactions.

A chain code is 32 bytes of key-derivation data associated with every extended key. It acts as additional pseudorandom input when generating child keys. It is not a private key by itself, but an xpub’s chain code must still be protected because it enables public child-key derivation.

With normal, or non-hardened, derivation, a child public key can be derived from the parent xpub.

With hardened derivation, the child depends on the parent private key rather than only on public information.
The corresponding child public key cannot be derived from the parent xpub. Hardened steps therefore provide stronger separation, but they prevent xpub-only derivation beyond that point.

Derivation paths use an apostrophe to mark hardened indexes:

m/84'/0'/0'/0/5

Here, 84', 0', and 0' are hardened, while 0 and 5 use normal derivation. A common design is to harden the purpose, coin type, and account levels, then use normal derivation for change and address indexes.