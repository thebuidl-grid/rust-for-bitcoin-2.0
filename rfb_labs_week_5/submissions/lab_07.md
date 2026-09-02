# Lab 07 — BIP39 mnemonic and seed

**Author:** [Christopher Dominic Eze](https://github.com/Christopherdominic)

## Commands used

```bash
cargo test --test lab_07
cargo run --example evidence   # scratch script, deleted after copying the output below
```

All of this used only the published class mnemonic:

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

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

```
MnemonicReport { word_count: 12, entropy_bits: 128, checksum_bits: 4 }
empty-passphrase seed:     5eb00bbddcf069084889a8ab9155568165f5c453ccb85e70811aaed6f6da5fc19a5ac40b389cd370d086206dec8aa6c43daea6690f20ad3d8d48b2d2ce9e38e4
"class"-passphrase seed:   49898e2ab94399fe78dee33d5e9e856e2b3b28827e95d8c125dd14c28cdaf181910c6fc1ac23c958f3263daa164d183eb5c2589a8ab530059ae6e8f67755d06a
seeds differ:               true
```

## Evidence references

- `src/labs/lab07_bip39.rs` — `inspect_mnemonic`, `mnemonic_seed_hex`,
  `compare_passphrases`, `is_public_test_mnemonic`.
- `tests/lab_07.rs::matches_the_published_bip39_seed_vector` — checks
  `mnemonic_seed_hex(MNEMONIC, "TREZOR")` against the well-known BIP39 test vector
  seed; that's the same "all abandon...about" mnemonic used with the reference
  trezor test-vector passphrase, and it passes.
- `tests/lab_07.rs::rejects_an_invalid_checksum` — the same 12 words with the last
  word swapped from `about` to `abandon` fails to parse, because that combination
  doesn't satisfy the checksum bits.

## Explanation

Entropy, checksum, mnemonic, seed, and passphrase are five different things and it's
easy to blur them together. Entropy is the actual randomness — for this mnemonic,
128 raw bits generated (in principle) from a secure RNG. The checksum is
`SHA256(entropy)`, truncated to `entropy_bits / 32` bits (4 bits here, since
128/32 = 4) and appended to the entropy before it's chopped into 11-bit chunks and
mapped to wordlist words. That's why 12 words encode 128 + 4 = 132 bits exactly
(132 / 11 = 12 words). The mnemonic is just those words in order — a
human-friendly re-encoding of entropy+checksum, nothing more.

The checksum's job is error detection, not encryption or security. If you mistype or
misremember a word, the checksum bits very likely won't match anymore and parsing
fails loudly, which is exactly what `rejects_an_invalid_checksum` demonstrates — swap
one word and the whole thing is rejected. But the checksum doesn't hide or protect
the entropy in any way; anyone who has the words has the entropy, full stop. It's a
typo-catcher, not a lock.

The seed is a completely different, much bigger thing: 512 bits produced by running
PBKDF2-HMAC-SHA512 over the mnemonic's normalized sentence (used as the password) and
`"mnemonic" + passphrase` (used as the salt), for 2048 rounds. The passphrase — often
called the "25th word" — isn't stored in the mnemonic anywhere and isn't derived from
it; it's a separate secret that gets mixed into the seed derivation. That's exactly
why `compare_passphrases` shows two completely different 64-byte seeds for the same
12 words: change the passphrase and PBKDF2's salt changes, so its output changes
completely, even though the entropy encoded in the words themselves is untouched.

That's also exactly why a forgotten passphrase can't be recovered from the mnemonic
alone: PBKDF2 is designed to be one-way, and the passphrase never gets written down
as part of the mnemonic or its checksum. If someone has the 12 words but not the
passphrase, they can derive the "no passphrase" wallet just fine, but the
passphrase-protected wallet is a different, unreachable seed to them — there's no
computation that gets you from the words back to a passphrase you never wrote down.
