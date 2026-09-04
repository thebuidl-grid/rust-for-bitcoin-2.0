# Lab 07 — BIP39 mnemonics, seeds, and passphrases

## Commands used

```
cargo test --test lab_07 -- --nocapture
cargo fmt --check
```

Implementation: `src/labs/lab07_bip39.rs` — `inspect_mnemonic`, `mnemonic_seed_hex`,
`compare_passphrases`, `is_public_test_mnemonic`. Shared parsing in
`src/labs/common.rs`.

## Terminal output

```
running 4 tests
test validates_entropy_and_checksum_structure ... ok
test rejects_an_invalid_checksum ... ok
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Observed behaviour with the public test mnemonic
`abandon abandon ... abandon about`:
- `inspect_mnemonic` = `{ word_count: 12, entropy_bits: 128, checksum_bits: 4 }`.
- 12x `abandon` (bad checksum) returns `Err(LabError::InvalidMnemonic(..))`.
- `mnemonic_seed_hex(m, "TREZOR")` = the published BIP39 vector
  `c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a6987599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04`.
- `compare_passphrases(m, "class")` yields two different 128-hex-char seeds.

## Evidence references

- `src/labs/lab07_bip39.rs` and `src/labs/common.rs` — use the `bip39` crate's
  `Mnemonic::parse` (checksum-checked) and `to_seed` (PBKDF2-HMAC-SHA512, 2048 rounds).
- `tests/lab_07.rs` — the published seed vector and the invalid-checksum rejection.
- Terminal block above from `cargo test --test lab_07`.

## Explanation

These five terms are distinct layers. **Entropy** is the raw random number (128
bits here). **Checksum** is the first `ENT/32` bits of `SHA256(entropy)` (4 bits),
appended so the total bit-length is a multiple of 11. **Mnemonic** is that
`entropy||checksum` blob sliced into 11-bit groups, each mapped to a word in the
2048-word list — so the words *encode* the entropy and let software detect typos
via the checksum. **Seed** is a different thing: `PBKDF2(password = mnemonic
sentence, salt = "mnemonic" + passphrase, 2048 iterations, HMAC-SHA512)` producing
512 bits, and it is the input to BIP32. **Passphrase** is the optional extra salt
("25th word"): it is never checked, never stored, and any value silently produces
a completely different valid seed and therefore a different wallet. That is why the
same words with `""` and with `"class"` give unrelated seeds, and why a wrong
passphrase looks like an empty wallet rather than an error.
