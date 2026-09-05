# Lab 07 — BIP39 mnemonic and seed

## Commands used

```bash
cargo test --test lab_07
cargo fmt --check
```

Only the public class mnemonic
(`abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about`)
and the well-known `"TREZOR"`/`"class"` test passphrases were used — no real recovery material.

## Terminal output

```text
$ cargo test --test lab_07
running 4 tests
test rejects_an_invalid_checksum ... ok
test validates_entropy_and_checksum_structure ... ok
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

`cargo fmt --check` produced no diff.

## Evidence references

- Implementation: `src/labs/lab07_bip39.rs`
- Test suite: `tests/lab_07.rs`
- `inspect_mnemonic` reports `word_count: 12`, `entropy_bits: 128`, `checksum_bits: 4` for the
  public test mnemonic, and returns an error for the 12-`abandon`-words variant whose last word
  fails the checksum.
- `mnemonic_seed_hex(MNEMONIC, "TREZOR")` reproduces the published BIP39 test vector seed
  byte-for-byte (`c55257c3...e7463b04`).
- `compare_passphrases` proves the empty-passphrase seed and the `"class"`-passphrase seed differ.

## Explanation

BIP39 has four distinct concepts that are easy to conflate. **Entropy** is the random bit string
that seeds everything (128 bits here). The **checksum** is `entropy_bits / 32` bits (4 bits for
128-bit entropy) taken from the leading bits of `SHA256(entropy)` and encoded into the final word;
it is error *detection*, not encryption — it lets software catch a mistyped or corrupted word
list, but it adds no secrecy and provides no protection against a stolen mnemonic. The **mnemonic**
is simply entropy+checksum re-encoded as human-readable words from a fixed wordlist — a lossless,
public transformation, so anyone who has the words can recover the same entropy. The **seed** is
the 512-bit value actually used to build the BIP32 master key, produced by PBKDF2-HMAC-SHA512 over
the mnemonic sentence with 2048 rounds and a salt of `"mnemonic" + passphrase`. That passphrase is
optional and *not* part of the mnemonic or its checksum at all — which is exactly why
`compare_passphrases` shows the same 12 words produce a completely different seed (and therefore a
completely different wallet) once a passphrase is added. Because the passphrase never gets
recorded anywhere the mnemonic is recorded, a forgotten passphrase is unrecoverable: there is no
checksum, hint, or derivation step that reveals it, unlike a mistyped word, which the checksum can
at least flag as wrong.

