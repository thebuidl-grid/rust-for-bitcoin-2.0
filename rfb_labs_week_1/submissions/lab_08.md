# Lab 08 — Block security

## Commands used

```bash
cargo test --test lab_08
```

RPC methods called:
- `getblockheader <hash>` - Retrieve block header with all cryptographic fields
- `generatetoaddress <count> <address>` - Mine multiple blocks for confirmation depth

## Terminal output

```
running 4 tests
test decodes_proof_linked_block_header ... ok
test mines_requested_confirmation_depth ... ok
test proves_one_confirmation_becomes_six ... ok
test reads_wallet_confirmation_depth ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All tests pass successfully, demonstrating:
- Parsing block header fields: hash, height, previous block hash, merkle root, nonce, difficulty, bits, confirmations, chainwork
- Mining additional blocks increases confirmation depth from 1 to 6
- Chainwork accumulates with each block, representing cumulative proof-of-work

## Explanation

Lab 08 examines block headers - the cryptographic proof layer of Bitcoin:

1. **Block Header Fields**:
   - **hash**: SHA256 double-hash of header (unique identifier)
   - **previousblockhash**: Hash of prior block (creates chain linkage)
   - **merkleroot**: Hash of all transactions in the block
   - **nonce**: Value miners adjust to find valid hash
   - **difficulty**: Target difficulty for this block
   - **bits**: Compact representation of difficulty target
   - **confirmations**: Number of blocks built on top
   - **chainwork**: Cumulative proof-of-work from genesis to this block

2. **Hash Linking**: Each block references the prior block's hash. This creates an immutable chain - altering any prior block changes its hash, breaking all subsequent links. This is why history becomes more secure with each block.

3. **Proof of Work**: The nonce field shows miners did computational work. Finding a valid nonce (hash < target) requires ~2^256/difficulty attempts. Accumulated difficulty across blocks creates exponential security.

4. **Confirmation Depth**: Transaction security grows with confirmation depth:
   - 1 confirmation: Miner invested in the block, but chainwork still low
   - 6 confirmations: ~10^20 hashes of work (with 2020 difficulty), making reorganization prohibitively expensive
   - Each additional block multiplies security exponentially

5. **Chainwork**: Measured in hex, this represents cumulative computational work from genesis. Higher chainwork = more secure, as reorganizing to a low-chainwork fork would require immense computational resources.
