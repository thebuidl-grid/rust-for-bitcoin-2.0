# Lab 01 — Address and network identification

## Commands used

TODO: List the Rust commands you ran.

```bash
cargo fmt
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --test lab_01
```

## Terminal output

TODO: Record the checked formats, networks, and scriptPubKeys.

The Lab 01 test suite completed successfully:

```bash
running 4 tests
test maps_regtest_prefixes ... ok
test identifies_human_readable_prefixes ... ok
test inspects_a_network_checked_address ... ok
test rejects_an_address_for_the_wrong_network ... ok

test result: ok. 4 passed; 0 failed

The implementation recognizes these address families:
```

- P2PKH: 1... on Bitcoin mainnet, m... or n... on testnet/regtest
- P2SH: 3... on Bitcoin mainnet, 2... on testnet/regtest
- P2WPKH: bc1q... on Bitcoin, tb1q... on testnet/signet, bcrt1q... on regtest
- P2TR: bc1p... on Bitcoin, tb1p... on testnet/signet, bcrt1p... on regtest

The implementation also converts a network-checked address into its corresponding scriptPubKey hex.

## Evidence references

TODO: Link screenshots or describe attached evidence.

The cargo test --test lab_01 terminal output above is the primary execution evidence. It demonstrates that all four required behaviors passed, including address-family identification, regtest prefix mapping, scriptPubKey inspection, and rejection of an address belonging to the wrong network.

## Explanation

TODO: Explain why prefix inspection alone is not complete address validation.

Bitcoin addresses have prefixes that give us a useful clue about what kind of address we are looking at. For example, 1... usually indicates a legacy P2PKH address, 3... indicates P2SH, bc1q... indicates native SegWit, and bc1p... indicates Taproot.

But the prefix by itself is not enough to prove that an address is valid. It does not check the rest of the address, including its checksum, and it does not prove that the address belongs to the network we intend to use. A valid Bitcoin mainnet address should not accidentally be accepted when our application is working with regtest.

The task, first identifies the likely address format from the prefix, but then actually parses the address with rust-bitcoin and requires it to belong to the requested network. Once the address has passed those checks, we can safely obtain its scriptPubKey.

The important idea is: the prefix helps us identify an address, but parsing and network validation are what make the result trustworthy.

