# Lab 07 — BIP39 mnemonic and seed

## Commands used

cargo fmt
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --test lab_07

All tests were run using only the public BIP39 test mnemonic:

abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about

No real wallet recovery phrases or private wallet secrets were used.

## Terminal output

running 4 tests
test rejects_an_invalid_checksum ... ok
test validates_entropy_and_checksum_structure ... ok
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

## Evidence references

The public Lab 07 test output above is the primary execution evidence.

The tests verify that:

- the public mnemonic contains 12 words
- it represents 128 bits of entropy with a 4-bit checksum
- an invalid checksum is rejected
- the mnemonic produces the published BIP39 seed for the `TREZOR` passphrase
- changing the passphrase produces a different seed

The public test mnemonic is intentionally used for this lab. No real wallet
recovery phrase should be included in source code, terminal output, screenshots,
or submissions.

## Explanation

### What is BIP39?

BIP39 defines a way to represent wallet backup information as a sequence of
human-readable words.

Instead of asking a person to write down a long string of random hexadecimal
data, BIP39 allows wallet entropy to be represented as a mnemonic sentence such
as:

    abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about

The mnemonic is designed to be easier for a human to back up and restore than
raw binary or hexadecimal data.

However, an important distinction is that the **mnemonic is not the wallet's
seed**.

The mnemonic is an intermediate representation that is used to derive the seed.

The basic flow is:

    entropy
       ↓
    checksum
       ↓
    mnemonic words
       ↓
    BIP39 seed
       ↓
    wallet key derivation

### Entropy and checksum

The test mnemonic contains 12 words.

BIP39 divides the mnemonic into two conceptual parts:

- entropy
- checksum

For a 12-word mnemonic, the entropy is 128 bits and the checksum is 4 bits.

The checksum length is calculated as:

    CS = ENT / 32

For 128 bits of entropy:

    CS = 128 / 32
       = 4 bits

The total number of bits is therefore:

    128 + 4 = 132 bits

BIP39 divides those 132 bits into groups of 11 bits. Each 11-bit value maps to
one word in the BIP39 word list:

    132 / 11 = 12 words

This explains why the public mnemonic contains exactly 12 words.

### Why does the checksum matter?

The checksum helps detect mistakes in a mnemonic.

The mnemonic:

    abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about

is valid because its checksum matches the entropy it represents.

If the final word is changed to produce:

    abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon

the checksum no longer matches.

The `rejects_an_invalid_checksum` test verifies that the implementation rejects
this mnemonic rather than treating every sequence of valid BIP39 words as a
valid recovery phrase.

This is useful when restoring a wallet because a typing or transcription error
can be detected before attempting key derivation.

### From mnemonic to seed

Once we have a valid mnemonic, BIP39 can derive a 512-bit seed from:

    mnemonic + passphrase

The passphrase is processed together with the mnemonic by the BIP39
mnemonic-to-seed function.

This means the mnemonic itself is not the final key material used by the wallet.
The resulting 512-bit seed becomes the starting point for later hierarchical key
derivation.

This is important because the next labs build on this seed to derive extended
keys and wallet addresses.

The `mnemonic_seed_hex` function performs this conversion and returns the
resulting 512-bit seed as hexadecimal.

### The passphrase is part of the wallet

A particularly important property of BIP39 is that the same mnemonic can produce
different seeds when different passphrases are used.

For example:

    mnemonic + ""
        ↓
    seed A

and:

    mnemonic + "class"
        ↓
    seed B

produce different seeds.

The mnemonic has not changed, but the passphrase has, so the resulting seed is
different.

The `passphrase_selects_a_different_wallet` test verifies exactly this behavior.

This means that a BIP39 passphrase is not simply a password used to unlock the
same seed. Changing it changes the derived seed itself and therefore leads to a
different wallet.

A person restoring a wallet therefore needs to know both the mnemonic and the
correct passphrase if a passphrase was used.

### Why does the published test vector matter?

The test:

    matches_the_published_bip39_seed_vector

uses the public mnemonic together with the passphrase:

    TREZOR

and compares the resulting seed against the published BIP39 test vector.

This is stronger than simply checking that the function returns a 64-byte value.
It verifies that the implementation produces the exact expected BIP39 seed for a
known input.

### Connecting this lab to the next labs

This lab establishes the beginning of the wallet key-generation process.

The flow is:

    mnemonic
       ↓
    BIP39 seed
       ↓
    extended private/public keys
       ↓
    derivation paths
       ↓
    child keys
       ↓
    Bitcoin addresses

Lab 07 focuses on the first part: safely validating a mnemonic and deriving the
BIP39 seed.

The later BIP32 and BIP44 labs build on this seed to derive the hierarchical key
structure used by HD wallets.

The key distinction to remember is:

    Entropy → creates the underlying randomness

    Checksum → helps detect mnemonic errors

    Mnemonic → human-readable representation of entropy + checksum

    Passphrase → additional input to seed derivation

    Seed → 512-bit result used as the starting point for hierarchical key
    derivation

A mnemonic therefore should not be thought of as "the private key." It is the
human-readable input from which the wallet's seed, and eventually its private
keys and addresses, are derived.