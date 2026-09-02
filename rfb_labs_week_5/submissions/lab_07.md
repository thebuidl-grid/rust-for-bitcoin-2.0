# Lab 07 — BIP39 mnemonic and seed

## Commands used

`cargo test --test lab_07`

## Terminal output

```bash
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running tests/lab_07.rs (target/debug/deps/lab_07-4f13bf30e901edac)

running 4 tests
test rejects_an_invalid_checksum ... ok
test validates_entropy_and_checksum_structure ... ok
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
``` 

## Evidence references

Code: src/labs/lab07_bip39.rs  
Test: tests/lab_07.rs

## Explanation

### Distinguish entropy, checksum, mnemonic, seed, and passphrase.

Entropy is the random input used to create a wallet. In BIP39, it is commonly 128, 160, 192, 224, or 256 bits.

The checksum is a small value calculated from the entropy and appended to it. Its purpose is to detect errors in the resulting word sequence.

The mnemonic is the human-readable list of words produced from the combined entropy and checksum. It represents the wallet backup, but it is not itself the binary seed.

The seed is derived from the mnemonic using a key-stretching function. Wallet software uses this seed to generate the master key and the wallet’s hierarchy of accounts and addresses.

The passphrase is an optional additional secret used together with the mnemonic when deriving the seed. Changing the passphrase produces a completely different seed and wallet, even when the mnemonic stays the same.