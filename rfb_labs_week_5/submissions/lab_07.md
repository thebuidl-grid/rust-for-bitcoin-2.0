# Lab 07 — BIP39 mnemonics, seeds, and passphrases

## Commands used

```bash
cargo test --test lab_07 -- --nocapture
```

## Terminal output

```
running 4 tests
test rejects_an_invalid_checksum ... ok
test validates_entropy_and_checksum_structure ... ok
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

## Evidence references

All four public tests in `tests/lab_07.rs` pass against `src/labs/lab07_bip39.rs`, using only the
published public test mnemonic (`abandon` x11 + `about`), never real recovery data:
`inspect_mnemonic` reports `word_count: 12`, `entropy_bits: 128`, `checksum_bits: 4`; a 12th-word
substitution that breaks the checksum (`... abandon abandon`) is correctly rejected by
`Mnemonic::parse`; `mnemonic_seed_hex(MNEMONIC, "TREZOR")` reproduces the published BIP39 test
vector seed byte-for-byte; and `compare_passphrases` proves an empty vs. `"class"` passphrase
produce two different 512-bit seeds from the same 12 words.

## Explanation

The BIP39 checksum is **error detection, not encryption**. It's derived by taking the first
`ENT/32` bits of `SHA256(entropy)` and appending them to the entropy before splitting the result
into 11-bit word indices (for 128 bits of entropy, that's 4 checksum bits, matching
`inspect_mnemonic`'s report). Its only job is to catch accidental mistakes — a mistyped word, a
transposed pair of words, a word dropped during copying — by making all-but-a-vanishingly-small
fraction of nearby "almost right" word sequences fail validation instead of silently producing a
different, wrong wallet. `rejects_an_invalid_checksum` demonstrates this: changing just the last
word to something with the right *length* but wrong *checksum bits* causes `Mnemonic::parse` to
fail outright, so an accidental transcription error is caught immediately rather than deriving a
different wallet.

Critically, the checksum reveals nothing about, and does not protect, the entropy itself — the
mnemonic words already *are* the entropy (re-encoded for human readability), so anyone who reads
the words has the whole secret; there is no additional passphrase-like protection from the
checksum. That protection instead comes from the **BIP39 passphrase**, an entirely optional 13th
factor that is never stored anywhere in the mnemonic or its checksum. `compare_passphrases` shows
that the same 12 words produce a *completely different* 512-bit seed depending on the passphrase
used (`""` vs. `"class"`) — the passphrase is mixed into the PBKDF2-HMAC-SHA512 derivation
alongside the mnemonic, not appended to or validated against anything. This is exactly why a
forgotten passphrase can never be recovered from the mnemonic alone: there is no artifact anywhere
(no hash, no checksum, no metadata) that records what the passphrase was or lets you verify a
guess without simply re-deriving the seed and checking whether the resulting wallet has the funds
you expect. The mnemonic's checksum only ever validates the 12 words against each other — it has
no relationship to the passphrase at all.
