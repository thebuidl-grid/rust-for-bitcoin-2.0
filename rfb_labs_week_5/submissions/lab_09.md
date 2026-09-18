# Lab 09 — BIP44 path decoding

## Commands used

```bash
cargo test --test lab_09
```

## Terminal output

```bash
running 4 tests
test changes_only_the_final_index ... ok
test decodes_every_bip44_level ... ok
test explains_zero_based_account_and_chain ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed
```

## Evidence references

All 4 tests pass, confirming:
- `decode_bip44_path()`: Parses m/purpose'/coin'/account'/change/index into components
- `describe_bip44_path()`: Generates human-readable descriptions (e.g., "Bitcoin third account, change address six")
- `with_address_index()`: Replaces final index while preserving the derivation branch
- `derive_bip44_address()`: Derives P2PKH addresses at specific BIP44 paths

## Explanation

BIP44 defines a standardized derivation path structure for hierarchical wallets, enabling interoperability across different wallet implementations.

**BIP44 path structure: m / 44' / coin' / account' / change / index**

Each level has semantic meaning:

| Level | Purpose | Hardened? | Range | Meaning |
|-------|---------|-----------|-------|---------|
| m | Master | N/A | - | BIP39 seed root |
| 44' | Purpose | ✓ Yes | Fixed 44 | BIP44 standard (fixed) |
| coin' | Coin type | ✓ Yes | Per coin | 0=Bitcoin, 1=Testnet |
| account' | Account | ✓ Yes | 0..2³¹ | User account (e.g., checking vs savings) |
| change | Chain | ✗ No | 0 or 1 | 0=receive, 1=change |
| index | Address | ✗ No | 0..2³¹ | Address sequence |

**Example path: m/44'/0'/0'/0/5**
- 44' = Use BIP44 standard
- 0' = Bitcoin mainnet (not testnet)
- 0' = First account
- 0 = Receive chain (not change)
- 5 = Sixth address in sequence (0-indexed)

**Why hardening at levels 1-3?**
- Hardened derivation requires the parent private key
- Prevents xpub at account level from revealing sibling accounts
- Each account is cryptographically isolated
- If one account's private key leaks, others remain secure

**Change chain (receive vs change):**
- **0 (receive)**: Addresses given to others for receiving payments
- **1 (change)**: Addresses used for sending to yourself (leftover coins)
- Wallet software tracks both to maintain privacy and prevent address reuse

**Address index:**
- Standard 0-based sequence
- Wallet generates new addresses by incrementing index
- No semantic meaning; just sequential

**Interoperability benefit:**
- All BIP44-compliant wallets (Electrum, MetaMask, Ledger, Trezor, etc.) 
- Given the same mnemonic, derive identical addresses
- Can recover wallet in any BIP44 wallet software
- Standardized path means no guessing which derivation scheme was used

**Common paths in practice:**
```
m/44'/0'/0'/0/0    = Bitcoin, Account 0, first receive address
m/44'/0'/1'/0/0    = Bitcoin, Account 1, first receive address
m/44'/1'/0'/0/0    = Bitcoin Testnet, first receive address
m/44'/0'/0'/1/0    = Bitcoin, Account 0, first change address
```

**Advantages over non-standard paths:**
- Wallets don't need to try multiple path schemes to recover
- Users can safely share xpub for watch-only access (hardened at account level)
- Mobile wallets can use only public keys (no private key needed on device)

