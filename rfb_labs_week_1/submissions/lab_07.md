# Lab 07 — Confirmation and block membership

## Commands used

```
cargo test --test lab_07

bitcoin-cli -regtest generatetoaddress 1 "<miner-address>"
bitcoin-cli -regtest getrawmempool
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "<txid>"
bitcoin-cli -regtest getblock "<block-hash>" 1
```

*RPCs are the ones issued by `mine_one_block`, `mempool_is_empty`, `transaction_confirmations`, and `confirm_and_locate_transaction` in `src/labs/lab07_confirm.rs`, verified against the mocked RPC client in `tests/lab_07.rs`. Run the `bitcoin-cli` lines against your live Polar regtest node to capture the terminal output below.*

## Terminal output

Captured against the live regtest node, confirming Lab 05's transaction:

```
$ bitcoin-cli -regtest generatetoaddress 1 "bcrt1qtdwur5a220ta9f0lndtdqf45f0kmaplt3xea5l"
[ "677f6328af7710f6b8fce2cac04a6c23eb2fe8dc2dacb73fd6896292992857c8" ]

$ bitcoin-cli -regtest getrawmempool
[
]

$ bitcoin-cli -regtest -rpcwallet=receiver gettransaction "7db84ed92ac38e4c6f01412011f9f97098cedd8a259c646c9cd1f192a9ff84c2"
{
  "amount": 1.00000000,
  "confirmations": 1,
  "blockhash": "677f6328af7710f6b8fce2cac04a6c23eb2fe8dc2dacb73fd6896292992857c8",
  "blockheight": 102,
  "blockindex": 1,
  ...
}

$ bitcoin-cli -regtest getblock "677f6328af7710f6b8fce2cac04a6c23eb2fe8dc2dacb73fd6896292992857c8" 1
{
  "hash": "677f6328af7710f6b8fce2cac04a6c23eb2fe8dc2dacb73fd6896292992857c8",
  "confirmations": 1,
  "height": 102,
  "nTx": 2,
  "previousblockhash": "20774b91b25a63e16a078d32fb2306c9461ff0bd51e22f673c3b9c4d96db5f7d",
  "tx": [
    "677758b6fcff8f926c543dff60bc269e07246395a682dd98d5424e507e240718",
    "7db84ed92ac38e4c6f01412011f9f97098cedd8a259c646c9cd1f192a9ff84c2"
  ]
}
```

After mining, `getrawmempool` is empty (the transaction left the mempool), `gettransaction` now reports `confirmations: 1` and a `blockhash`, and `getblock`'s `tx` array (block 102) contains our txid alongside the block's own coinbase transaction — direct proof the transaction is inside that block.

## Evidence references

Evidence is the live terminal output above, captured directly via `docker exec bitcoind-lab-a bitcoin-cli ...` against a real regtest node (not a screenshot — this session ran headlessly, no Polar GUI was open).

## Explanation

Mining one block changed four concrete, observable things:

1. **It left the mempool.** `getrawmempool` went from listing the txid to returning an empty array — once a transaction is included in a block, it's no longer "waiting," so nodes drop it from their mempool.
2. **`confirmations` went from `0` to `1`.** This field literally counts how many blocks — including the one containing the transaction itself — sit on top of it. Before mining, zero blocks contained it; after, exactly one does.
3. **A `blockhash` and `blockheight` appeared.** `gettransaction` now reports which specific block (`677f6328...`, height 102) the transaction was mined into — information that doesn't exist for a purely mempool transaction, since it isn't attached to the chain yet.
4. **It became part of that block's `tx` array.** `getblock ... 1` on block 102 lists every transaction inside it — the block's own coinbase (`677758b6...`) and our payment (`7db84ed9...`). This `tx` array is the direct, verifiable evidence the transaction is now actually in the blockchain, not just claimed to be.

Nothing about the transaction's *content* (its inputs, outputs, signatures) changed — mining doesn't alter transactions. What changed is its *status*: it moved from "broadcast and pending" to "permanently recorded," with the block acting as its receipt.
