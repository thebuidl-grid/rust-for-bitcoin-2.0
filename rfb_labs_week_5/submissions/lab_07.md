# Lab 07 — BIP39 mnemonic and seed

## Commands used

```
cargo test --test lab_07 -- --nocapture
```

## Terminal output

```
running 4 tests
test rejects_an_invalid_checksum ... ok
test validates_entropy_and_checksum_structure ... ok
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

## Evidence references

Public 12-word class test mnemonic: `abandon abandon abandon abandon abandon abandon
abandon abandon abandon abandon abandon about`.

```
inspect_mnemonic:  MnemonicReport { word_count: 12, entropy_bits: 128, checksum_bits: 4 }
```

Swapping the final word to `abandon` (12x `abandon`, no valid checksum word) makes
`inspect_mnemonic` return `Err` — the checksum catches the corrupted word list.

With the published BIP39 test-vector passphrase `"TREZOR"`, the derived seed matches the
publicly documented vector exactly:

```
c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a6987599d18264c1e1
c92f2cf141630c7a3c4ab7c81b2f001698e7463b04
```

Comparing an empty passphrase against the disposable test passphrase `"class"` on the
same mnemonic produced two different 64-byte seeds (`seeds_differ: true`), proving the
passphrase selects an entirely different wallet even though the recovery words are
identical.

## Explanation

Entropy is the raw random bits chosen when the mnemonic was generated (128 bits for a
12-word mnemonic). The checksum is `ENT / 32` bits (4 bits here) taken from the SHA-256
hash of that entropy and appended before splitting into 11-bit word indices — its only
job is to catch transcription mistakes (a wrong word, a wrong word order, a dropped word)
by making almost all invalid word combinations fail the checksum check. It is not
encryption and provides no secrecy: anyone who has the words already has the entropy in
plaintext, checksum bits included. The mnemonic (the words) encodes only the entropy; the
seed is a completely different value produced by running PBKDF2-HMAC-SHA512 over the
mnemonic string and an optional passphrase, 2048 rounds, into 64 bytes. Because the
passphrase is mixed into the seed derivation, it is not stored anywhere and never appears
in the mnemonic itself — a forgotten passphrase cannot be recovered or brute-forced back
out of the mnemonic alone (short of exhausting the passphrase's own search space), since
the mnemonic's checksum only validates the words, not the passphrase.
