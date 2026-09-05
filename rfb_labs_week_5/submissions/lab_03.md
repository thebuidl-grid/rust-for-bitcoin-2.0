# Lab 03 — P2SH 2-of-3 multisig

## Commands used

```bash
cargo test --test lab_03
```

## Terminal output

```bash
running 4 tests
test builds_the_outer_p2sh_lock ... ok
test derives_the_committed_p2sh_address ... ok
test builds_a_two_of_three_redeem_script ... ok
test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed
```

## Evidence references

All 4 tests pass, confirming:
- `build_2_of_3_redeem_script()`: Creates 2-of-3 multisig script with OP_2 <pubkey1> <pubkey2> <pubkey3> OP_3 OP_CHECKMULTISIG
- `derive_p2sh_address()`: Hashes the redeem script and encodes as P2SH address
- `build_p2sh_script_pubkey()`: Builds OP_HASH160 <script_hash> OP_EQUAL script
- `inspect_p2sh_multisig()`: Reports redeem script, address, and scriptPubKey together

## Explanation

P2SH (Pay-to-Script-Hash) enables flexible spending conditions without exposing the script until spend time.

**How P2SH works:**
1. Redeem script is written (e.g., 2-of-3 multisig): `2 <pub1> <pub2> <pub3> 3 CHECKMULTISIG`
2. Redeem script is hashed: `scriptHash = HASH160(redeem_script)`
3. P2SH address encodes this hash (similar to P2PKH address format)
4. Receiver publishes address, not the script

**Locking script (scriptPubKey):**
```
OP_HASH160 <script_hash> OP_EQUAL
```

**Unlocking (spending):**
1. Spender provides signatures and the original redeem script
2. Script execution: verify redeem script hashes to the locked hash
3. Then execute the redeem script with provided signatures

**Two-of-Three Multisig example:**
```bash
2 <pubkey1> <pubkey2> <pubkey3> 3 OP_CHECKMULTISIG
```
Requires any 2 of the 3 keys to authorize a spend.

**Advantages of P2SH:**
- Receiver doesn't need to know the full script in advance
- Spender pays for script complexity (script size affects fees)
- Flexible script types: multisig, time-locks, conditional spends
- Address size is fixed regardless of script complexity

**Disadvantages:**
- Spender must know the redeem script (must be communicated separately)
- Larger scriptSig during spending (includes script and signatures)

