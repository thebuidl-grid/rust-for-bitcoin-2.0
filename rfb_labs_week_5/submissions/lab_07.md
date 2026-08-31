# Lab 07 — BIP39 mnemonic and seed

## Commands used

I formatted the source, ran the focused BIP39 tests, and then ran the complete Lab 7
suite using only the published disposable test mnemonic:

```bash
cargo fmt
cargo test --test lab_07 validates_entropy_and_checksum_structure
cargo test --test lab_07 rejects_an_invalid_checksum
cargo test --test lab_07 matches_the_published_bip39_seed_vector
cargo test --test lab_07 passphrase_selects_a_different_wallet
cargo test --test lab_07
```

## Terminal output

The public 12-word vector produced the expected structure without exposing any real
wallet secret:

```text
word_count: 12
entropy_bits: 128
checksum_bits: 4
empty and protected passphrase seeds differ: true

test result: ok. 4 passed; 0 failed
```

## Evidence references

- Implementation: `src/labs/lab07_bip39.rs`
- Public tests: `tests/lab_07.rs`
- `inspect_mnemonic` validates the English wordlist and checksum before reporting
  the mnemonic structure.
- `mnemonic_seed_hex` matches the published BIP39 vector using the public mnemonic
  and `TREZOR` test passphrase.
- No real mnemonic, passphrase, seed, or private key was used or recorded.

## Explanation

Entropy is the cryptographically random starting value. BIP39 appends a short
checksum derived from the entropy and divides the combined bits into 11-bit indexes
into a 2,048-word list. The resulting ordered words are the mnemonic, commonly
called a recovery or seed phrase. For 12 words, 128 entropy bits plus 4 checksum bits
produce 132 bits, which divide into twelve 11-bit word indexes.

The checksum detects some transcription errors; it does not encrypt the mnemonic or
prevent theft. The mnemonic and optional passphrase are processed with the BIP39
seed function to produce a 512-bit seed. The seed is binary key-generation material,
not the words themselves. An empty passphrase is valid, and every different
passphrase creates a different valid seed and wallet. The passphrase is not encoded
in the mnemonic, so forgetting it prevents recovery of the intended wallet even if
the words are correct.
