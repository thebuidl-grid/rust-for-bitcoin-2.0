# Lab 07 — BIP39 mnemonic and seed

## Commands used

```shell
test@pop-os:~/Desktop/rust/rust-for-bitcoin-2.0/rfb_labs_week_5$ cargo test --test lab_07
```

Only the public class test mnemonic (`abandon` ×11 + `about`) and the published
`TREZOR` test vector were used.

## Terminal output

```terminaloutput
running 4 tests
test rejects_an_invalid_checksum ... ok
test validates_entropy_and_checksum_structure ... ok
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

For the 12-word mnemonic, `inspect_mnemonic` reports `word_count: 12`,
`entropy_bits: 128`, `checksum_bits: 4`. `is_public_test_mnemonic` returns true.
`compare_passphrases` confirms the seed computed with an empty passphrase differs
from the seed computed with the protected passphrase (`seeds_differ: true`).
The seed derivation against `"TREZOR"` matches the published BIP39 test vector.
(No secret material is reproduced here.)

## Evidence references

```
Code: src/labs/lab07_bip39.rs
Test: tests/lab_07.rs
```

## Explanation

BIP39 separates **entropy**, **checksum**, **mnemonic**, **seed**, and **passphrase**.

- **Entropy (ENT)** is the underlying random data the user intends to back up — here
  128 bits for a 12-word sentence. Entropy of `ENT` bits is split into 11-bit groups,
  each mapped to one word from the English word list.
- **Checksum (CS)** is the first `ENT/32` bits of the `SHA256` of the entropy
  (`128/32 = 4` bits). It is appended to the entropy so the total is divisible by 11,
  and it lets a wallet detect a mistyped word. The checksum is **error detection,
  not encryption** — it adds no secrecy and cannot reconstruct missing entropy; it
  only verifies that the sentence is internally consistent.
- **Mnemonic** is the human-readable encoding of `ENT + CS`. It faithfully
  represents the entropy, so anyone with the words (and passphrase) can regenerate
  the wallet.
- **Seed** is the 512-bit output of the PBKDF2 key-stretching function
  (`PBKDF2-HMAC-SHA512`, 2048 iterations) over the mnemonic plus a salt containing a
  **passphrase**.
- **Passphrase** is an optional second factor. It is not stored in the words; the
  same mnemonic with a different (or empty) passphrase yields an entirely different
  seed and therefore an entirely different wallet (`seeds_differ: true`). Because the
  passphrase is never recoverable from the mnemonic, a forgotten passphrase means the
  associated wallet cannot be regenerated — there is no way to brute-force it back
  from the words alone.
