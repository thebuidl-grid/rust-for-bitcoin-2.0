# Lab 08 — Block security

## Commands used

TODO: Record block-header inspection and additional mining commands.
# 1. Fetch verbose block header details for a target block hash
bitcoin-cli -regtest getblockheader "00000000001a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c"

# 2. Check initial transaction confirmation count (expect 1)
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "payment-txid"

# 3. Mine 5 additional blocks to build proof-of-work depth on top of the block
bitcoin-cli -regtest generatetoaddress 5 "bcrt1qmineraddress"

# 4. Verify updated transaction confirmation depth (expect 6 confirmations)
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "payment-txid"

## Terminal output

TODO: Show header fields and confirmation count changing from one to six.
$ cargo test --test lab_08
running 4 tests
test decodes_proof_linked_block_header ... ok
test mines_requested_confirmation_depth ... ok
test proves_one_confirmation_becomes_six ... ok
test reads_wallet_confirmation_depth ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

{
  "header": {
    "hash": "block-hash",
    "height": 102,
    "previousblockhash": "previous-hash",
    "merkleroot": "merkle-root",
    "nonce": 7,
    "difficulty": 0.00000001,
    "bits": "207fffff",
    "confirmations": 1,
    "chainwork": "00000000000000ce"
  },
  "confirmations_before": 1,
  "confirmations_after": 6
}...

## Evidence references

% TODO: Link screenshots or describe the attached evidence.

Automated Unit Tests: Verified via cargo test --test lab_08 passing all 4 tests:
1. decodes_proof_linked_block_header: Validates fetching and parsing getblockheader output into BlockHeaderEvidence.
2. mines_requested_confirmation_depth: Verifies mining $N$ additional blocks using generatetoaddress.
3. reads_wallet_confirmation_depth: Confirms parsing transaction status from gettransaction to extract confirmation count.
4. proves_one_confirmation_becomes_six: Validates the full sequence proving that mining 5 additional blocks increases confirmation depth from 1 to 6.

## Explanation

TODO: Explain hash links, Merkle roots, proof of work, and confirmation depth.

1. Proof-Linked Block Headers:
- A Bitcoin block header contains critical metadata linking it to the global consensus chain: previousblockhash (creates an immutable hash chain back to genesis), merkleroot (commits to all transactions in the block), bits/difficulty (target work threshold), and nonce (proof-of-work solution).
- chainwork records the total expected number of hashes performed across the entire chain history leading to this block.
2. Confirmation Depth & Reorg Security:1 
- Confirmation: The transaction is included in block $H$. To double-spend it, an attacker must rewrite block $H$.
- 6 Confirmations: 5 additional blocks ($H+1$ through $H+5$) have been built on top of block $H$. To invalidate the transaction now, an attacker must replace $H$ and outperform the honest network in producing 6 consecutive blocks.
- As additional blocks are mined on top of the transaction's block, the probability of an attacker successfully executing a chain reorganization drops exponentially, reaching practical finality for standard transactions at 6 confirmations.
