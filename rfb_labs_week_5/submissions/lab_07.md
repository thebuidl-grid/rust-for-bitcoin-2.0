# Lab 07 — BIP39 mnemonic and seed

## Commands used

```bash
cargo test --test lab_07
cargo fmt --check
bash grader/grade.sh
```

## Terminal output

```text
running 4 tests
test rejects_an_invalid_checksum ... ok
test validates_entropy_and_checksum_structure ... ok
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

## Evidence references

Automated unit tests in `tests/lab_07.rs` verifying 12-word BIP39 mnemonic validation (128 bits entropy + 4 bits checksum), checksum error detection on invalid word lists, published test vector seed generation with passphrase `"TREZOR"`, and passphrase wallet isolation. Uses only the public test mnemonic: `abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`.

## Explanation

BIP39 maps raw binary entropy to human-readable word lists and derives a cryptographic seed using the following concepts:
- **Entropy & Checksum**: 128 bits of random entropy are generated and hashed with SHA-256. The first 4 bits of the hash are appended as a checksum (`CS = ENT / 32`), creating a 132-bit sequence. The checksum provides error detection when restoring a wallet to prevent typos.
- **Mnemonic Words**: The 132 bits are split into 12 11-bit groups (`2^11 = 2048`), mapping each group to an index in the standard 2048-word English dictionary.
- **Seed Derivation**: The mnemonic string is converted into a 512-bit binary seed via PBKDF2-HMAC-SHA512 using 2,048 hashing rounds and `"mnemonic" + passphrase` as the salt.
- **Passphrase**: An optional passphrase acts as a 13th word. Because it is part of the PBKDF2 salt, a different passphrase produces a completely different 512-bit seed and HD wallet branch from the exact same mnemonic phrase. A forgotten passphrase cannot be recovered from the mnemonic alone.
