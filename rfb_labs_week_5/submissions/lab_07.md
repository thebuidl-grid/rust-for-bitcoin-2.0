# Lab 07 — BIP39 mnemonic and seed

## Commands used

```bash
cargo test --test lab_07 -- --nocapture
cargo run --example labs_demo
```

## Terminal output

```text
$ cargo test --test lab_07 -- --nocapture
running 4 tests
test rejects_an_invalid_checksum ... ok
test validates_entropy_and_checksum_structure ... ok
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

```text
$ cargo run --example labs_demo   (Lab 07 section, public test mnemonic only)
inspect_mnemonic(PUBLIC_TEST_MNEMONIC) = Ok(MnemonicReport { word_count: 12, entropy_bits: 128, checksum_bits: 4 })
is_public_test_mnemonic = true
mnemonic_seed_hex(MNEMONIC, "TREZOR") = Ok("c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a
    6987599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04")
compare_passphrases(MNEMONIC, "class") = PassphraseComparison {
    empty_passphrase_seed_hex: "5eb00bbddcf069084889a8ab9155568165f5c453ccb85e70811aaed6f6da5fc
        19a5ac40b389cd370d086206dec8aa6c43daea6690f20ad3d8d48b2d2ce9e38e4",
    protected_seed_hex:        "49898e2ab94399fe78dee33d5e9e856e2b3b28827e95d8c125dd14c28cdaf1
        81910c6fc1ac23c958f3263daa164d183eb5c2589a8ab530059ae6e8f67755d06a",
    seeds_differ: true }
```

## Evidence references

- Implementation: [`src/labs/lab07_bip39.rs`](../src/labs/lab07_bip39.rs)
- Public test suite: [`tests/lab_07.rs`](../tests/lab_07.rs) — 4/4 passing, logged in
  [`grading/logs/lab_07.log`](../grading/logs/lab_07.log).
- The `TREZOR`-passphrase seed above matches the well-known published BIP39 test vector
  for the all-`abandon` + `about` mnemonic.
- Only the public 12-word test mnemonic (`abandon` × 11 + `about`) and disposable demo
  passphrases (`"TREZOR"`, `"class"`) are used anywhere in this lab. No real recovery
  material appears in this file or the source.

## Explanation

BIP39 has four distinct pieces that are easy to blur together:

- **Entropy** is the raw randomness the mnemonic encodes — for our 12-word mnemonic,
  128 bits (`entropy_bits`). It is what actually needs to be unpredictable; everything
  else is derived from it.
- **Checksum** is `entropy_bits / 32` extra bits (4 bits here, `checksum_bits`) appended
  to the entropy before it is split into 11-bit word indices, computed as the leading
  bits of SHA256(entropy). It exists purely for **error detection**: if a word is
  mistyped or two words are swapped, the checksum almost certainly no longer matches,
  and `Mnemonic::parse` — used in `inspect_mnemonic` — rejects it, which is exactly what
  `rejects_an_invalid_checksum` demonstrates by flipping the last word. It is *not*
  encryption: the checksum bits are derived from, and are recoverable from, the
  entropy itself, so knowing the checksum does not protect the entropy in any way —
  anyone who reads all 12 words already has the full entropy in the clear.
- **Mnemonic** is the human-readable 12-to-24-word encoding of entropy + checksum. It is
  what people write down and what `inspect_mnemonic`/`is_public_test_mnemonic` operate
  on directly.
- **Seed** is the 512-bit output of PBKDF2-HMAC-SHA512 over the mnemonic's normalized
  words (2048 rounds) with an optional **passphrase** mixed in as the salt
  (`"mnemonic" + passphrase`). This is what `mnemonic_seed_hex` computes and what BIP32
  actually uses as its master key material (Lab 08) — the mnemonic itself is never used
  directly for key derivation, only its seed is.

Because the passphrase is folded into the seed derivation itself rather than being
checked against anything stored in the mnemonic or its checksum, there is no way to
recover a forgotten passphrase from the mnemonic alone — `compare_passphrases` shows
the empty-passphrase seed and the `"class"`-passphrase seed are two completely
different, equally valid-looking 64-byte seeds, with nothing in either seed or the
mnemonic revealing which passphrase (if any) produced it. This is a deliberate BIP39
feature (sometimes called a "25th word"): it lets one set of 12/24 words open different
wallets depending on the passphrase, which is powerful for plausible-deniability setups
but means a forgotten passphrase is unrecoverable, not merely inconvenient — there is no
checksum or hint anywhere to test candidate passphrases against except by deriving
addresses and checking on-chain for known funds.
