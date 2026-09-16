# Lab 02 — Legacy P2PKH

## Commands used

```bash
cargo test --test lab_02
```

## Terminal output

```bash
running 4 tests
test commits_to_hash160_of_the_public_key ... ok
test puts_unlocking_data_in_scriptsig ... ok
test builds_the_standard_p2pkh_lock ... ok
test derives_the_expected_p2pkh_address ... ok

test result: ok. 4 passed; 0 failed
```

## Evidence references

All 4 tests pass, confirming:
- `derive_p2pkh_address()`: Creates P2PKH addresses from hex-encoded public keys
- `build_p2pkh_script_pubkey()`: Constructs the OP_DUP OP_HASH160 <hash> OP_EQUALVERIFY OP_CHECKSIG script
- `committed_pubkey_hash()`: Returns the HASH160 of the public key
- `p2pkh_spend_template()`: Models ScriptSig items (signature and pubkey) with empty witness

## Explanation

P2PKH (Pay-to-Public-Key-Hash) is Bitcoin's original address format. It combines two-layer hashing and scripting:

**Locking (receiving):**
1. User's public key is hashed with HASH160 (RIPEMD160 of SHA256)
2. Address encodes this 20-byte hash in base58check format
3. Receiving address = Base58Check(version_byte + hash160 + checksum)

**Locking script (scriptPubKey):**
```bash
OP_DUP OP_HASH160 <pubkey_hash> OP_EQUALVERIFY OP_CHECKSIG
```

**Unlocking (spending):**
1. Spender provides their signature and public key in ScriptSig
2. Script execution: verify public key hashes to the locked hash, then verify signature

**Advantages:**
- Address is shorter (20 bytes vs 33 bytes for full pubkey)
- Public key is only revealed during spend, not at address generation

**Disadvantages:**
- Two hash operations in every spend verification
- Vulnerable to future quantum key recovery (addresses reveal pubkey when spent)
