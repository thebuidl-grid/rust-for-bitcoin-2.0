# Lab 08 — BIP32 extended keys

## Commands used

```bash
cargo test --test lab_08
```

## Terminal output

```bash
running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed
```

## Evidence references

All 4 tests pass, confirming:
- `master_xpriv()`: Creates master xpriv from BIP39 seed
- `derive_extended_keys()`: Derives xpriv/xpub pair at any path
- `derive_normal_child_xpub()`: Derives public child keys without private key material
- `path_contains_hardened_step()`: Detects hardened indices in derivation paths

## Explanation

BIP32 defines Hierarchical Deterministic (HD) wallets where all keys derive from a single master seed using a cryptographic tree structure.

**Extended Keys (xpriv / xpub):**

Extended keys encode both the key material and the chain code (additional entropy):
- **xpriv**: Extended private key (serialized: `xprv...`)
  - Contains: version + depth + parent fingerprint + index + chain code + private key
  - Starts with `xprv` on mainnet, `tprv` on testnet
  - Can derive all child keys (both hardened and normal)
  
- **xpub**: Extended public key (serialized: `xpub...`)
  - Contains: version + depth + parent fingerprint + index + chain code + public key
  - Starts with `xpub` on mainnet, `tpub` on testnet
  - Can only derive normal (non-hardened) child keys

**Chain Code:**
- 32 bytes of extra entropy included with every key
- Prevents attackers who know a public key from deriving child keys
- Required for proper BIP32 derivation

**Derivation types:**

| Type | Index range | Private → Public? | Public → Child? | Use case |
|------|-------------|------------------|-----------------|----------|
| Normal | 0 to 2³¹-1 | ✓ Yes (via chain code) | ✓ Yes | Standard derivation |
| Hardened | 2³¹ to 2³²-1 | ✓ Yes | ✗ No | Account separation |

**Hardened derivation:**
- Indexed as m/i' where i' = i + 2³¹
- Requires private key (even from xpub, cannot derive hardened child)
- Breaking private key at one level doesn't reveal sibling branches
- Used for account-level separation (each account is hardened)

**Normal derivation:**
- Standard indices (m/i where i < 2³¹)
- Can be derived from xpub alone (public derivation)
- Enables watch-only wallets (observe balances without spending)
- Used for address sequences within accounts

**Derivation path example: m/44'/0'/0'/0/0**
```bash
m              = master (seed)
44'            = BIP44 purpose (hardened, requires xpriv)
0'             = Bitcoin coin type (hardened, requires xpriv)
0'             = Account 0 (hardened, requires xpriv)
0              = External chain (normal, can use xpub)
0              = Address 0 (normal, can use xpub)
```

**Key property:**
Every key carries its complete derivation information, allowing wallets to:
- Export watch-only versions (xpub only, cannot spend)
- Derive specific branches without knowing parent secrets
- Restore entire key trees from master seed

