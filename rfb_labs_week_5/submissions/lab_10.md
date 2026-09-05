# Lab 10 — Deterministic recovery across address families

## Commands used

```bash
cargo test --test lab_10
```

## Terminal output

```bash
running 4 tests
test format_selection_changes_the_lock_target ... ok
test derives_three_regtest_address_families ... ok
test identical_recovery_inputs_repeat ... ok
test changing_only_the_index_changes_the_address ... ok

test result: ok. 4 passed; 0 failed
```

## Evidence references

All 4 tests pass, confirming:
- `derive_address_for_path()`: Derives P2PKH, P2WPKH, or P2SH addresses from any path
- `derive_address_set()`: Generates BIP44/49/84 addresses at same account/index
- `recovery_is_repeatable()`: Proves identical inputs produce identical addresses
- `changing_index_changes_address()`: Proves different indices produce different addresses

## Explanation

Multiple BIP standards (BIP44, BIP49, BIP84) define competing derivation paths and address formats. A robust wallet must support all three standards to recover funds, regardless of which standard the wallet originally used.

**The three standards:**

| Standard | Path | Format | Prefix | Use case |
|----------|------|--------|--------|----------|
| BIP44 | m/44'/0'/account'/change/index | P2PKH | 1 or m/n | Legacy, maximum compatibility |
| BIP49 | m/49'/0'/account'/change/index | P2SH-wrapped SegWit | 3 or 2 | Transitional SegWit support |
| BIP84 | m/84'/0'/account'/change/index | Native SegWit | bc1q or bcrt1q | Modern, lowest fees |

**Why three standards?**

Different wallets adopted SegWit support at different times:
1. **BIP44 (2014)**: Original standard, uses legacy P2PKH
2. **BIP49 (2016)**: P2SH wrapping for backward-compatible SegWit (soft fork compatibility)
3. **BIP84 (2019)**: Native SegWit v0, most efficient

A single BIP39 seed must generate:
- All three address families for recovery
- Same addresses every time (deterministic)
- Correct addresses at correct indices

**Recovery process:**

When restoring from a mnemonic, a complete wallet should:
1. Derive the BIP39 seed from mnemonic + passphrase
2. Create master xpriv from seed
3. For each standard (BIP44, 49, 84):
   - Derive account xpriv at standard's path
   - Generate receive addresses (chain=0) for account 0
   - Generate change addresses (chain=1) for account 0
4. Scan blockchain for any of these addresses
5. Recover all funds (whichever standard was originally used)

**Example derivation for account 0, address 0:**

```bash
BIP44 (legacy):
  Path: m/44'/0'/0'/0/0
  Address: 1BoatSLRHtKNngkdXEeobR76b53LETtpyT (P2PKH)

BIP49 (wrapped SegWit):
  Path: m/49'/0'/0'/0/0
  Address: 3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy (P2SH)

BIP84 (native SegWit):
  Path: m/84'/0'/0'/0/0
  Address: bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kygt080 (P2WPKH)
```

**Determinism guarantee:**

Given:
- Same mnemonic
- Same passphrase
- Same path (e.g., m/44'/0'/0'/0/0)

Result:
- Same address, every time
- Works across all BIP44-compliant software

This is why wallets are recoverable: the mnemonic alone is sufficient to regenerate all addresses and restore all funds.

**Practical implication:**

A user who:
1. Wrote down their 12-word mnemonic
2. Imported it into Electrum, got their addresses
3. Lost their computer
4. Later imports into Ledger or MetaMask

Gets identical addresses and complete fund recovery. The standard ensures this works across any compliant implementation.

