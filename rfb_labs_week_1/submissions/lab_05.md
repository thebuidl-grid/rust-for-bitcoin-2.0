# Lab 05 — Broadcast and mempool

## Commands used

- Payment: `bitcoin-cli -rpcwallet=miner sendtoaddress "$RECEIVER_ADDRESS" 1`
- Mempool: `bitcoin-cli getrawmempool`
- Sender transaction: `bitcoin-cli -rpcwallet=miner gettransaction "$TXID"`
- Receiver pending balance: `bitcoin-cli -rpcwallet=receiver getbalances`

## Terminal output

```text
txid: fed36157b3cb634faf2ba9fb29adb0a8e6316599e59af7bb1f78db250f4ad070
mempool_contains_tx: true

sender_status:
  confirmations: 0
  amount: -1.0 BTC
  fee: -0.00002820 BTC
  block_hash: null

receiver_balance:
  trusted: 0.0 BTC
  untrusted_pending: 1.0 BTC
  immature: 0.0 BTC
```

## Evidence references
![alt text](evidence/image-3.png)

## Explanation

A built transaction specifies its inputs and outputs; signing supplies the
authorization needed to spend those inputs. Broadcasting sends the signed
transaction to a node or its peers. A valid unconfirmed transaction can then reside
in each node's local mempool, but mempool membership is not global consensus. It
becomes confirmed only when a miner includes it in a valid block accepted into the
active chain.
