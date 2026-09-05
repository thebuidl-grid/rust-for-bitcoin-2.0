# Lab 04 — Native P2WPKH

## Commands used

```bash
cargo test --test lab_04
```

## Terminal output

```bash
running 4 tests
test leaves_scriptsig_empty_and_uses_witness ... ok
test derives_a_native_regtest_address ... ok
test builds_a_version_zero_witness_lock ... ok
test reports_a_twenty_byte_program ... ok

test result: ok. 4 passed; 0 failed
```

## Evidence references

All 4 tests pass, confirming:
- `derive_p2wpkh_address()`: Creates native SegWit addresses from compressed public keys
- `build_p2wpkh_script_pubkey()`: Constructs version-0 witness script: `0 <20-byte-hash>`
- `witness_program()`: Reports witness version (0) and program (pubkey hash)
- `native_spend_template()`: Demonstrates empty scriptSig with witness items

## Explanation

P2WPKH (Pay-to-Witness-Public-Key-Hash) is native SegWit v0, introduced in BIP141. It separates transaction signature data into a separate "witness" section, reducing the data size for transaction size calculation.

**Key differences from P2PKH:**

| Aspect | P2PKH | P2WPKH |
|--------|-------|--------|
| scriptPubKey | OP_DUP OP_HASH160 <hash> OP_EQUALVERIFY OP_CHECKSIG | 0 <hash> |
| ScriptSig | Contains signature and pubkey | Empty |
| Signature data location | scriptSig (counts as 4x bytes) | Witness (counts as 1x bytes) |
| Address prefix | 1 (mainnet), m/n (testnet) | bc1q (mainnet), bcrt1q (regtest) |

**Witness program (0 <20-byte-hash>):**
- Version 0 indicates SegWit v0
- 20-byte program is the HASH160 of the public key (same as P2PKH)
- Compact encoding saves ~25% transaction size due to witness discount

**Why ScriptSig is empty:**
- Before SegWit, scriptSig had to contain all data needed to validate spending
- SegWit moved signature/pubkey data to the witness field
- scriptSig is kept empty to maintain backward compatibility with pre-SegWit nodes
- Full validation nodes check both scriptPubKey and witness

**Advantages:**
- ~25% smaller transactions (witness data counts as 1 byte, not 4)
- Lower fees for the same number of inputs/outputs
- Fixes transaction malleability (witness not included in txid)
- Enables advanced features like payment channels

**Disadvantages:**
- Requires SegWit-compatible wallet (not all wallets support yet)
- Some older services may not recognize witness addresses


