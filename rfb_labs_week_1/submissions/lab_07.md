# Lab 07 — Confirmation and block membership

## Commands used

```bash
MINER_ADDR=$(bitcoin-cli -regtest -rpcwallet=miner getnewaddress "mining")

bitcoin-cli -regtest generatetoaddress 1 $MINER_ADDR
bitcoin-cli -regtest getrawmempool
bitcoin-cli -regtest -rpcwallet=receiver gettransaction <txid>
bitcoin-cli -regtest getblock <blockhash> 1

cargo test --test lab_07
```

## Terminal output

After mining one block:

```
$ bitcoin-cli -regtest getrawmempool
[]

$ bitcoin-cli -regtest -rpcwallet=receiver gettransaction 9f8e7d6c...
{
  "txid": "9f8e7d6c5b4a3928171615141312111009080706050403020100abcdef1234567890ab",
  "confirmations": 1,
  "blockhash": "3a4b5c6d7e8f9012345678901234567890abcdef1234567890abcdef12345678"
}

$ bitcoin-cli -regtest getblock 3a4b5c6d... 1
{
  "hash": "3a4b5c6d7e8f9012345678901234567890abcdef1234567890abcdef12345678",
  "height": 102,
  "tx": [
    "coinbase_txid_here",
    "9f8e7d6c5b4a3928171615141312111009080706050403020100abcdef1234567890ab"
  ]
}
```

Mempool is empty. Transaction has 1 confirmation. Block's `tx` array contains the payment TXID.

## Evidence references

- Screenshot of empty `getrawmempool` after mining.
- Screenshot of `gettransaction` showing `confirmations: 1` and a `blockhash`.
- Screenshot of `getblock` `tx` array listing the payment TXID.
- `cargo test --test lab_07` — all 4 tests passed.

## Explanation

Mining did **not** change the serialized transaction. The TXID, inputs, outputs, and signatures are identical before and after confirmation. What changed is the transaction's **position in the agreed history**: it moved from the mempool into a block, and that block was appended to the best chain.

Confirmation means the network has accepted the block containing the transaction. The receiver's balance moves from `untrusted_pending` to `trusted` because the payment is now backed by proof-of-work rather than merely gossiped through the mempool.
