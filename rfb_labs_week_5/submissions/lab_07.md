# Lab 07 — BIP39 mnemonic and seed

## Commands used

```bash
cargo test --test lab_07
bash grader/grade.sh
```

## Terminal output

running 4 tests

test rejects_an_invalid_checksum ... ok

test validates_entropy_and_checksum_structure ... ok

test matches_the_published_bip39_seed_vector ... ok

test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

## Evidence references

All four public tests in `tests/lab_07.rs` pass, using only the published
public BIP39 test mnemonic (`abandon abandon abandon abandon abandon
abandon abandon abandon abandon abandon abandon about`):
- Validating the 12-word mnemonic and reporting 128 bits of entropy and 4
  checksum bits (`validates_entropy_and_checksum_structure`)
- Rejecting a mnemonic with the same word count but an invalid final-word
  checksum (`rejects_an_invalid_checksum`)
- Deriving the 512-bit seed with the `"TREZOR"` passphrase and matching it
  against the officially published BIP39 test vector
  (`matches_the_published_bip39_seed_vector`)
- Confirming the public test mnemonic is recognized, and that a different
  passphrase (`"class"`) produces a different seed than no passphrase
  (`passphrase_selects_a_different_wallet`)


## Explanation

A BIP39 mnemonic encodes two distinct things packed together: **entropy**
(the actual randomness the wallet is built from) and a **checksum** (a few
bits derived by hashing that entropy, appended so it can be verified
later). For a 12-word mnemonic, that's 128 bits of entropy plus 4 checksum
bits, packed into 132 bits total and split into eleven-bit chunks that map
to word-list indices.

The checksum is error detection, not encryption: it doesn't hide, encrypt,
or protect the entropy in any way — the entropy is fully recoverable
directly from the words themselves once decoded. All the checksum does is
let software confirm that the words were transcribed, typed, or read back
correctly, since a single typo or transposed word will, with high
probability, produce a checksum mismatch and get rejected (`rejects_an_invalid_checksum`
demonstrates exactly this). Nothing about the checksum's presence protects
the mnemonic from anyone who reads it.

The **seed** is a further derivation: 512 bits produced by running the
mnemonic words and an optional **passphrase** through PBKDF2-HMAC-SHA512.
Unlike the checksum, the passphrase genuinely changes the outcome — it
isn't validated or checked against anything, it's mixed directly into the
key-derivation function's salt. This means the same 12 words with two
different passphrases produce two completely different, unrelated 512-bit
seeds (as `passphrase_selects_a_different_wallet` proves) — effectively two
different wallets from the same mnemonic.

This is exactly why a forgotten passphrase cannot be recovered from the
mnemonic alone: the mnemonic only reconstructs entropy, and the checksum
only verifies that reconstruction is textually correct. Neither the words
nor their checksum contain any information about what passphrase, if any,
was used — PBKDF2 is a one-way function, so there is no way to work
backward from a derived seed (or from the funds/addresses it controls) to
recover which passphrase produced it. If the passphrase is lost, the
wallet built from `mnemonic + passphrase` is permanently unrecoverable,
even with the mnemonic words fully intact.

