# Lab 07 — BIP39 mnemonics, seeds, and passphrases

## Commands used

```bash
cargo test --test lab_07 -- --nocapture
cargo fmt --check
cargo clippy --all-targets
```

Implementation lives in `src/labs/lab07_bip39.rs`: `inspect_mnemonic`,
`mnemonic_seed_hex`, `compare_passphrases`, and `is_public_test_mnemonic`,
using only the published class mnemonic:

```
abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

## Terminal output

```
running 4 tests
test rejects_an_invalid_checksum ... ok
test validates_entropy_and_checksum_structure ... ok
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

`matches_the_published_bip39_seed_vector` checks `mnemonic_seed_hex(MNEMONIC,
"TREZOR")` against the well-known public BIP39 test vector seed
(`c55257c360c07c72...e7463b04`), confirming the PBKDF2-HMAC-SHA512 derivation
is byte-for-byte correct.

## Evidence references

- `src/labs/lab07_bip39.rs` — `inspect_mnemonic` reports 12 words, 128 bits of
  entropy, and 4 checksum bits (`ENT / 32` per BIP39); `mnemonic_seed_hex`
  derives the 512-bit seed.
- `tests/lab_07.rs` — `rejects_an_invalid_checksum` feeds all-`abandon`
  13 words (a valid word list but wrong checksum) and confirms
  `inspect_mnemonic` returns an error.
- `bash grader/grade.sh` recorded `07 | 4/4 | 4 | ...` for this lab.

## Explanation

BIP39's checksum is 4 bits taken from the SHA-256 hash of the entropy and
appended before splitting into 11-bit word indices — it exists purely so a
wallet can detect a wrong or mistyped word list before deriving any keys from
it, the same role a check-digit plays on a credit card number. It has no
encryption property at all: the checksum is derived deterministically from
entropy that is otherwise stored (via the word list) in plain sight, so
anyone can recompute and verify it, and doing so reveals nothing extra about
the entropy that the words themselves didn't already reveal.
`rejects_an_invalid_checksum` demonstrates the detection side directly — the
mnemonic decodes into valid English words, so it is not gibberish, but its
last word encodes the wrong final checksum bits and `inspect_mnemonic`
correctly refuses it.

The optional BIP39 passphrase is the actual secret-holding step, and it is
never stored anywhere recoverable by design. `compare_passphrases` shows that
feeding the same 12 words through PBKDF2-HMAC-SHA512 with `""` versus a
protected passphrase produces two completely different 512-bit seeds — the
passphrase is mixed into the salt of that KDF, not appended to or encrypted
alongside the mnemonic, so there is no ciphertext to decrypt and no reverse
operation from seed back to passphrase. If a wallet was funded under a
passphrase that is forgotten, the 12 words alone regenerate the *empty*
passphrase wallet, a different, unfunded wallet by construction — the funds
are not "hidden" behind the passphrase in the way a password protects an
encrypted file, they are at an entirely different, unrelated derivation path
that has no algorithmic link back to the mnemonic without the exact
passphrase string.
