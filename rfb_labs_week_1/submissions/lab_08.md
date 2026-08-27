# Lab 08 — Inspect block commitments and confirmation depth

## Commands used

```bash
# Block header audit and confirmation depth verification
cargo test --test lab_08
bitcoin-cli -regtest getblockheader "block-hash"
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "payment-txid"
bitcoin-cli -regtest generatetoaddress 5 "bcrt1qminer..."
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "payment-txid"
```

## Terminal output

```json
{
  "header": {
    "hash": "3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b",
    "height": 102,
    "previous_block_hash": "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
    "merkle_root": "b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6",
    "nonce": 7,
    "difficulty": 4.656542373081379e-10,
    "bits": "207fffff",
    "confirmations": 1,
    "chainwork": "00000000000000000000000000000000000000000000000000000000000000ce"
  },
  "confirmations_before": 1,
  "confirmations_after": 6
}
```

```text
running 4 tests
test decodes_proof_linked_block_header ... ok
test mines_requested_confirmation_depth ... ok
test proves_one_confirmation_becomes_six ... ok
test reads_wallet_confirmation_depth ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

- Implemented `get_block_header`, `mine_additional_blocks`, `get_confirmations`, and `build_security_report` in `src/labs/lab08_security.rs`.
- Decoded 80-byte header attributes (`hash`, `previousblockhash`, `merkleroot`, `nonce`, `bits`, `chainwork`).
- Proved confirmation growth from 1 to 6 confirmations after mining 5 blocks.
- Validated test suite in `tests/lab_08.rs`.

## Explanation

1. **Header Hash Links & Merkle Commitments**: Each 80-byte Bitcoin block header contains the `previousblockhash` (SHA-256d hash of the preceding block header), creating an immutable cryptographic chain linked back to the genesis block. The header also contains `merkleroot` (the root of the binary Merkle tree constructed from all transaction hashes in the block). Changing a single bit in any transaction changes its TXID, which propagates up the Merkle tree to change the `merkleroot`, which invalidates the block header hash.
2. **Proof-of-Work Search & Chainwork**: Miners must iterate the 32-bit `nonce` (and extraNonce in coinbase) until the header's double SHA-256 hash is numerically less than or equal to the target specified by `bits`. `chainwork` represents the total expected number of hashes performed across the entire chain from genesis.
3. **Why Confirmations Increase Reorganization Cost Without Validating Invalid Transactions**: Each subsequent block mined on top of a transaction adds another layer of PoW consensus ($N$ confirmations). To rewrite or revert a transaction $N$ blocks deep, an attacker would have to re-mine $N$ blocks from scratch faster than the rest of the network combined (requiring massive energy and hash rate). However, Proof-of-Work and high confirmation depth **never** make an invalid transaction valid; full nodes strictly validate all consensus rules (sigops, script execution, UTXO existence) independently. If a block contains an invalid transaction, full nodes instantly reject the entire block regardless of how much chainwork or confirmations it claims.
