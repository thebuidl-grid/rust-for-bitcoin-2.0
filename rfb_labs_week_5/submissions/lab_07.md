# Lab 07 — BIP39 mnemonic and seed

## Commands used

`cargo test --test lab_07` using only the published `abandon ... about` test mnemonic.

## Terminal output

All 4 tests passed. The mnemonic has 12 words, 128 entropy bits, and 4 checksum bits. The published seed vector matched, invalid checksum words were rejected, and a passphrase changed the seed.

## Evidence references

Evidence: terminal output from `cargo test --test lab_07`. No private or production wallet material was used or recorded.

## Explanation

Entropy is the initial random bit string. The checksum is appended before splitting into words. The mnemonic encodes both parts; BIP39 combines those words with a passphrase to derive the seed. The passphrase is additional wallet input.

