# Lab 07 — BIP39 mnemonic and seed

## Commands used

```bash
cargo test --test lab_07
cargo run -- 7
cargo run --example bip_vectors
```

The runner validates the published class mnemonic, reports its entropy and checksum
sizes, shows the checksum rejecting a tampered final word, and derives three seeds from
the same twelve words under three passphrases. The example compares the derived seed
against the published BIP39 vector.

## Terminal output

```text
$ cargo test --test lab_07
running 4 tests
test rejects_an_invalid_checksum ... ok
test validates_entropy_and_checksum_structure ... ok
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

$ cargo run -- 7
== Lab 07: BIP39 mnemonics and seeds ==
  published test mnemonic recognized: true
  12 words | 128 entropy bits | 4 checksum bits
  last word swapped to 'abandon': rejected: invalid BIP39 mnemonic: the mnemonic has an invalid checksum
  seed with TREZOR passphrase (BIP39 vector)
    c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a6987599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04
  seed with no passphrase
    5eb00bbddcf069084889a8ab9155568165f5c453ccb85e70811aaed6f6da5fc19a5ac40b389cd370d086206dec8aa6c43daea6690f20ad3d8d48b2d2ce9e38e4
  seed with "class" passphrase
    49898e2ab94399fe78dee33d5e9e856e2b3b28827e95d8c125dd14c28cdaf181910c6fc1ac23c958f3263daa164d183eb5c2589a8ab530059ae6e8f67755d06a
  seeds differ: true
```

## Evidence references

Implementation in `src/labs/lab07_bip39.rs`, tests in `tests/lab_07.rs`, runner in
`src/main.rs` under `lab07`, vector check in `examples/bip_vectors.rs`.

Everything here uses the published `abandon ... about` test mnemonic, which is known to
everyone and must never receive real funds. No personal wallet material appears
anywhere in this branch.

The TREZOR seed `c55257c360c07c72...698e7463b04` is the official BIP39 English test
vector for the all-zero entropy, and `cargo run --example bip_vectors` confirms it
independently. The empty-passphrase seed `5eb00bbddcf06908...2ce9e38e4` is the one
BIP49 and BIP84 build their own vectors on, which is why Labs 08 to 10 reproduce their
published addresses.

Changing the final word from `about` to `abandon` leaves twelve words that are all in
the BIP39 list, and it still fails the checksum.

## Explanation

Five terms in this lab get confused with each other.

Entropy is the raw randomness, 128 bits for twelve words. The checksum is the first
ENT/32 bits of SHA256(entropy), so 4 bits here. Appending it gives 132 bits, which
divides into twelve groups of 11 bits, and each group indexes a word in the 2048 word
list. The mnemonic is that word encoding and nothing else. The seed is a separate
512-bit value from PBKDF2-HMAC-SHA512 over the normalized words, salted with the string
"mnemonic" plus the passphrase, run 2048 times. The passphrase is an optional input to
that function.

The checksum detects errors. It does not encrypt or protect anything. Four bits means a
random twelve word sequence from the list passes about one time in sixteen. What it
catches is the realistic failure, a mistyped or transposed word during transcription,
and that is what it was designed for. It does not authenticate the mnemonic and it does
not hide it. Anyone holding the twelve words holds the wallet. The output shows the
useful half of this, where swapping `about` for `abandon` gives twelve valid dictionary
words that BIP39 still rejects.

A forgotten passphrase cannot be recovered from the mnemonic, because it is never
stored. It is not encrypted by the mnemonic and not hinted at by it, and there is no
field in the backup holding it. The passphrase enters only as part of the PBKDF2 salt,
and PBKDF2 runs one way. Each distinct passphrase gives a different 512-bit seed, so a
different master key and a different wallet. The three seeds in the output show it: the
same twelve words with no passphrase, with TREZOR and with `class` produce three
unrelated wallets.

There is no such thing as an invalid passphrase, which cuts both ways. A typo silently
opens a different empty wallet instead of raising an error, and that is what makes
plausible deniability work. It also means recovery has to reproduce the passphrase
exactly, including case and whitespace, and losing it loses the funds as completely as
losing the mnemonic. The checksum covers the words only, so no software can tell you
the passphrase was wrong.
