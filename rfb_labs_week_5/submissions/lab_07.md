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

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Evidence references

`src/labs/lab07_bip39.rs`, using only the public test mnemonic (12× "abandon"..."about"):

- `inspect_mnemonic` calls `bip39::Mnemonic::parse`, which fails on a bad checksum
  (proven by `rejects_an_invalid_checksum`, which swaps the final word to another
  "abandon" and gets an error). For the valid mnemonic: `word_count = 12`,
  `entropy_bits = 128` (from `to_entropy().len() * 8`), `checksum_bits = 4`
  (`entropy_bits / 32`, per BIP39's `CS = ENT / 32`).
- `mnemonic_seed_hex` calls `Mnemonic::to_seed(passphrase)` (PBKDF2-HMAC-SHA512, 2048
  rounds) and matches the published BIP39 test vector for this mnemonic with passphrase
  `"TREZOR"`.
- `compare_passphrases` derives seeds with `""` and a supplied passphrase and reports
  `seeds_differ = true` — a different passphrase is a completely different 512-bit
  seed, not a variation of the same one.
- `is_public_test_mnemonic` normalizes whitespace and checks against the known
  `"abandon" x11 + "about"` string, so this file never risks handling anything but the
  published test vector.

## Explanation

The checksum is the first `ENT/32` bits of SHA256(entropy), appended before the
words get split out. Its whole job is catching a mistyped or misordered word when
someone's reading a mnemonic back off paper — same role a check digit plays on a
credit card number. Nothing more. It's fully public and reversible from the words
themselves, so it can't hide or protect anything; anyone who sees a valid BIP39
mnemonic can recompute its checksum trivially.

The passphrase is a completely different mechanism — it never touches the checksum
or the word list at all. BIP39 feeds it straight into PBKDF2:
`PBKDF2-HMAC-SHA512(mnemonic, "mnemonic" + passphrase, 2048)`. It only exists in
whoever's memory holds it. Lose it, and there's nothing to check a guess against —
no stored hash, no checksum, nothing — except re-deriving the seed and looking for
funds on-chain. Every possible passphrase produces an equally valid-looking wallet
from the same mnemonic; there's no way to tell from the mnemonic alone which one (if
any) was actually used.
