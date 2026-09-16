# Lab 01 — Address and network identification

## Commands used

```bash
cargo test --test lab_01
```

## Terminal output

```bash
running 4 tests
test identifies_human_readable_prefixes ... ok
test maps_regtest_prefixes ... ok
test inspects_a_network_checked_address ... ok
test rejects_an_address_for_the_wrong_network ... ok

test result: ok. 4 passed; 0 failed
```

## Evidence references

All 4 tests pass, confirming:
- `identify_prefix()`: Correctly maps address strings to AddressFormat (P2PKH/P2SH/P2WPKH/P2TR)
- `expected_prefix()`: Returns correct prefixes for each format on each network
- `inspect_address()`: Parses and validates addresses match the expected network
- `script_pubkey_hex()`: Converts addresses to their scriptPubKey hex encoding

## Explanation

Bitcoin addresses use human-readable prefixes to encode both format and network information:

**Format identification by prefix:**
- `1` or `m/n`: P2PKH (Pay-to-Public-Key-Hash) - legacy format
- `3` or `2`: P2SH (Pay-to-Script-Hash) - multisig/wrapped formats
- `bc1q` or `bcrt1q` or `tb1q`: P2WPKH (native SegWit v0)
- `bc1p` or `bcrt1p` or `tb1p`: P2TR (Taproot v1)

**Network identification by prefix:**
- Mainnet: `1`, `3`, `bc1`
- Testnet: `m/n`, `2`, `tb1`
- Regtest: `m/n`, `2`, `bcrt1`

Prefix inspection alone is insufficient for address validation because:
1. A correctly formatted prefix doesn't guarantee the address is valid
2. An address with the right prefix for one network must be rejected if used on another network
3. The scriptPubKey (which commits to the actual public key or script) must be validated
4. Checksum validation (bech32 or base58check) is needed to catch transmission errors

The `inspect_address()` function combines prefix checking with network validation to ensure addresses are used in their correct context.
