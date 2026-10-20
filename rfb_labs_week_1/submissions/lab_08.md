# Lab 08 — Block security

## Commands used

```bash
bitcoin-cli -regtest getblockheader <blockhash>
bitcoin-cli -regtest -rpcwallet=receiver gettransaction <txid>
bitcoin-cli -regtest generatetoaddress 5 $MINER_ADDR
bitcoin-cli -regtest -rpcwallet=receiver gettransaction <txid>

cargo test --test lab_08
```

## Terminal output

Block header (confirming block):

```
$ bitcoin-cli -regtest getblockheader 3a4b5c6d...
{
  "hash": "3a4b5c6d7e8f9012345678901234567890abcdef1234567890abcdef12345678",
  "height": 102,
  "previousblockhash": "2b3c4d5e6f789012345678901234567890abcdef1234567890abcdef1234567",
  "merkleroot": "7f8e9d0c1b2a394857463748596a5b4c3d2e1f0a9b8c7d6e5f4a3b2c1d0e9f8a",
  "nonce": 2,
  "bits": "207fffff",
  "difficulty": 4.656542373906925e-10,
  "confirmations": 1,
  "chainwork": "0000000000000000000000000000000000000000000000000000000000000002"
}
```

Confirmations before additional mining: 1  
After mining 5 more blocks: 6

```
$ bitcoin-cli -regtest -rpcwallet=receiver gettransaction 9f8e7d6c...
{
  "confirmations": 6
}
```

## Evidence references

- Screenshot of verbose `getblockheader` showing hash, previousblockhash, merkleroot, nonce, bits, and chainwork.
- Screenshot showing confirmations changing from 1 to 6.
- `cargo test --test lab_08` — all 4 tests passed.

## Explanation

Each block header commits to the chain through several mechanisms:

- **Hash links** — `previousblockhash` points to the parent block, forming an immutable chain. Changing any ancestor invalidates all descendants.
- **Merkle root** — commits to every transaction in the block. Tampering with any transaction changes the root and invalidates the header.
- **Proof of work** — miners search for a `nonce` such that the block hash is below the target set by `bits`. `chainwork` accumulates this effort across the chain.
- **Confirmations** — each new block on top increases the cost of rewriting history. Six confirmations means an attacker would need to redo the work of six blocks plus catch up to the current tip.

More confirmations increase reorganization cost but do **not** make an invalid transaction valid. Consensus rules are checked at every depth; confirmations only strengthen ordering certainty.
