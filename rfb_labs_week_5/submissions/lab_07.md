# Lab 07 — BIP39 mnemonic and seed

## Commands used

```bash
cargo test --test lab_07
```

## Terminal output

```bash
running 4 tests
test rejects_an_invalid_checksum ... ok
test validates_entropy_and_checksum_structure ... ok
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed
```

## Evidence references

All 4 tests pass, confirming:
- `inspect_mnemonic()`: Validates mnemonic and reports entropy/checksum structure
- `mnemonic_seed_hex()`: Derives 512-bit seed from words + passphrase using PBKDF2
- `compare_passphrases()`: Demonstrates that different passphrases produce different seeds
- `is_public_test_mnemonic()`: Recognizes "abandon abandon ... about" (the test vector)

## Explanation

BIP39 defines a standard for converting random entropy into human-readable recovery words and deriving cryptographic keys from them.

**The chain: Entropy → Mnemonic → Seed → Keys**

**Entropy:**
- Random bits generated during wallet creation
- Standard sizes: 128 bits (12 words), 160 bits (15 words), 192 bits (18 words), 224 bits (21 words), 256 bits (24 words)
- Example: 128-bit entropy

**Checksum:**
- Hash of entropy to detect typos in the mnemonic
- Size = entropy_bits / 32
- Example: 128-bit entropy → 4-bit checksum

**Mnemonic:**
- Entropy + checksum bits grouped as 11-bit indices into BIP39 English word list (2048 words)
- Example: 128-bit entropy + 4-bit checksum = 132 bits = 12 × 11-bit words = 12 words
- Words are easy to write down, remember (partially), and speak

**Seed:**
- Generated from mnemonic using PBKDF2-HMAC-SHA512
- Formula: `seed = PBKDF2(mnemonic, "mnemonic" + passphrase, 2048 iterations)`
- Output: 512 bits (64 bytes)
- Same mnemonic always produces the same seed (for same passphrase)
- Used to derive all keys in the wallet (BIP32 master key)

**Passphrase (optional salt):**
- Additional secret material, optional, not part of the mnemonic
- Different passphrase → completely different seed → completely different keys
- Receiver doesn't need passphrase; mnemonic alone recovers base wallet
- Passphrase acts as "hidden account" protection

**Word count and entropy relationship:**
```bash
12 words  → 128-bit entropy
15 words  → 160-bit entropy
18 words  → 192-bit entropy
21 words  → 224-bit entropy
24 words  → 256-bit entropy
```

**Security implications:**
- 12 words = 128-bit entropy = adequate for most users
- 24 words = 256-bit entropy = quantum-resistant target
- Mnemonic order matters (reordering words produces invalid result)
- Single typo in word list lookup fails (word must exist and must be correct index)

