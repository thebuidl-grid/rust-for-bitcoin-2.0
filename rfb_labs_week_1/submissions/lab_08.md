# Lab 08 — Inspect block commitments and confirmation depth

## Commands used

```bash
# 1. Query verbose block header for confirming block
bitcoin-cli getblockheader "3c3df961b9eaf0f36914768b176805e190fa90e19432c7d8a72e9fb616f5e842" true

# 2. Query initial confirmation count (1 confirmation)
bitcoin-cli -rpcwallet=receiver gettransaction "42df86320b309d52b5f12402d7a3b90dabe933e12800303b63067bfe8537d4d1"

# 3. Mine 5 additional blocks to reach 6 confirmations
bitcoin-cli generatetoaddress 5 "bcrt1qmineraddress"

# 4. Re-query confirmation count (6 confirmations)
bitcoin-cli -rpcwallet=receiver gettransaction "42df86320b309d52b5f12402d7a3b90dabe933e12800303b63067bfe8537d4d1"

# 5. Run Rust tests for Lab 08
cargo test --test lab_08
```

## Terminal output

```text
$ bitcoin-cli getblockheader "3c3df961b9eaf0f36914768b176805e190fa90e19432c7d8a72e9fb616f5e842" true
{
  "hash": "3c3df961b9eaf0f36914768b176805e190fa90e19432c7d8a72e9fb616f5e842",
  "confirmations": 6,
  "height": 102,
  "version": 536870912,
  "merkleroot": "4a9c77d061c8afd92c9895937389d6c04281800bdedd1a44d9211772dc81b343",
  "time": 1740000000,
  "mediantime": 1740000000,
  "nonce": 0,
  "bits": "207f6679",
  "difficulty": 4.656542373081379e-10,
  "chainwork": "00000000000000000000000000000000000000000000000000000000000000ce",
  "previousblockhash": "42df86320b309d52b5f12402d..."
}

$ cargo test --test lab_08
running 4 tests
test decodes_proof_linked_block_header ... ok
test mines_requested_confirmation_depth ... ok
test proves_one_confirmation_becomes_six ... ok
test reads_wallet_confirmation_depth ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

![Polar Block Header & Security Depth Screenshot](evidence/lab06_10.png)

## Explanation

**Header Cryptographic Commitments & Confirmation Security:**
- **Hash Links**: Every block header contains the SHA-256 hash of the `previousblockhash`, forming an immutable cryptographic chain back to the Genesis block.
- **Merkle Root**: A 32-byte cryptographic root hash committing to all transactions in the block. Modifying any transaction changes its TXID, altering the Merkle root and invalidating the block header hash.
- **Proof-of-Work**: The `nonce` and `bits` fields represent the computational work spent satisfying the difficulty target.
- **Confirmation Depth & Reorg Cost**: As additional blocks are mined on top of the block containing a transaction, the confirmation count increases. Reverting a transaction with 6 confirmations requires an attacker to re-mine 6 consecutive blocks with greater total chainwork than the honest network. Confirmations increase the economic cost of re-mining history, but cannot make an invalid transaction valid (consensus rules are always strictly enforced).
