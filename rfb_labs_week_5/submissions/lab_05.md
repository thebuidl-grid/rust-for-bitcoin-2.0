# Lab 05 — Address compatibility map

## Commands used

```bash
cargo test --test lab_05
```

## Terminal output

```bash
running 4 tests
test names_the_required_human_encoding ... ok
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok
test builds_the_four_format_map ... ok
test selects_the_most_modern_supported_format ... ok

test result: ok. 4 passed; 0 failed
```

## Evidence references

All 4 tests pass, confirming:
- `can_send_to()`: Maps AddressFormat to required sender capabilities
- `compatibility_report()`: Builds full compatibility matrix for all formats
- `best_supported_format()`: Prefers P2TR → P2WPKH → P2SH → P2PKH in order
- `required_encoding()`: Returns "Base58Check", "Bech32", or "Bech32m" per format

## Explanation

Bitcoin address formats evolved over time, and not all wallets support all formats. Understanding compatibility is critical for ensuring payments reach their destination.

**Encoding schemes:**
- **Base58Check** (P2PKH, P2SH): Used by pre-SegWit wallets, human-readable but longer
- **Bech32** (P2WPKH, P2SH-wrapped SegWit): SegWit v0, more efficient, better error detection
- **Bech32m** (P2TR): Taproot (v1+), most recent standard

**Wallet capabilities over time:**

| Wallet Era | Supports | Cannot send to |
|------------|----------|-----------------|
| Pre-2012 (before P2SH) | P2PKH only (1...) | 3..., bc1q..., bc1p... |
| 2012-2017 (P2SH era) | P2PKH, P2SH (3...) | bc1q..., bc1p... |
| 2017+ (Native SegWit) | P2PKH, P2SH, P2WPKH (bc1q...) | bc1p... |
| 2021+ (Taproot support) | All formats (P2TR bc1p...) | - |

**Why P2SH wallets reject native SegWit (bc1q...):**
- Native SegWit addresses use bech32 encoding, not base58check
- Pre-SegWit wallets don't understand the `bc1q` prefix
- They cannot construct a transaction to an unknown address format
- This is a wallet/parser limitation, not a protocol issue

**Best practice selection:**
For maximum compatibility when sending, choose the most modern format the recipient's wallet supports:
1. **P2TR (bc1p...)** - Best fees, best privacy (if recipient supports)
2. **P2WPKH (bc1q...)** - Good fees, widely supported
3. **P2SH (3...)** - Broad compatibility, slightly higher fees
4. **P2PKH (1...)** - Universal support, highest fees

**Fee comparison at same network:**
- P2WPKH: ~25% smaller than P2PKH
- P2SH (wrapped): Same size as P2PKH but enables upgradability
- P2TR: ~15% smaller than P2WPKH (signature aggregation)

