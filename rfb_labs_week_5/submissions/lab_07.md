# Lab 07 — BIP39 mnemonic and seed

## Commands used

```
cargo test --test lab_07 -- --nocapture
```

Ad-hoc check against the public BIP39 test mnemonic (`abandon x11 about`) only:

```rust
lab07_bip39::inspect_mnemonic(MNEMONIC)
lab07_bip39::mnemonic_seed_hex(MNEMONIC, "TREZOR")
```

## Terminal output

```
running 4 tests
test rejects_an_invalid_checksum ... ok
test validates_entropy_and_checksum_structure ... ok
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

```
MnemonicReport { word_count: 12, entropy_bits: 128, checksum_bits: 4 }
seed (TREZOR passphrase) = c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a6987599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04
```

This matches the published BIP39 test vector for this mnemonic with passphrase
`TREZOR`. The 12-word all-`abandon`-plus-`about` mnemonic with the trailing word
changed to any other valid word (invalid checksum) is correctly rejected by
`inspect_mnemonic`.

## Evidence references

- `cargo test --test lab_07` output above.
- Source: `src/labs/lab07_bip39.rs`.
- Test suite: `tests/lab_07.rs`.
- Only the published public test mnemonic was used; no real mnemonic, passphrase, or
  seed was generated or recorded.

## Explanation

A BIP39 mnemonic encodes entropy (ENT) plus a checksum (CS) derived from
`SHA256(entropy)`, where `CS = ENT / 32` and the total bit length `ENT + CS` is split
into 11-bit word indices. For this 12-word mnemonic, `ENT = 128` bits and
`CS = 4` bits. The checksum's only job is error detection: it lets software reject a
mistyped or reordered word list before it silently derives the wrong wallet, but it is
not encryption and provides no confidentiality. The optional passphrase (BIP39's
"25th word") is combined with the mnemonic during the seed-derivation step (PBKDF2
over the normalized mnemonic and `"mnemonic" + passphrase`) to produce the 512-bit
seed; `compare_passphrases` shows that the empty-passphrase seed and the
`"class"`-passphrase seed are completely different, unrelated byte strings. Because
the passphrase is never stored anywhere, not in the mnemonic, not in any derived key,
a wallet protected by a forgotten passphrase cannot be recovered from the mnemonic
alone: the mnemonic reproduces every possible passphrase's family of wallets, but
only the one matching the original passphrase text is the wallet that was actually
funded.
