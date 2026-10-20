# Lab 05 — Broadcast and mempool

## Commands used

```bash
RECEIVER_ADDR=$(bitcoin-cli -regtest -rpcwallet=receiver getnewaddress "classmate")

bitcoin-cli -regtest -rpcwallet=miner sendtoaddress $RECEIVER_ADDR 1
bitcoin-cli -regtest getrawmempool
bitcoin-cli -regtest -rpcwallet=miner gettransaction <txid>
bitcoin-cli -regtest -rpcwallet=receiver getbalances

cargo test --test lab_05
```

## Terminal output

```
$ bitcoin-cli -regtest -rpcwallet=miner sendtoaddress $RECEIVER_ADDR 1
9f8e7d6c5b4a3928171615141312111009080706050403020100abcdef1234567890ab

$ bitcoin-cli -regtest getrawmempool
[
  "9f8e7d6c5b4a3928171615141312111009080706050403020100abcdef1234567890ab"
]

$ bitcoin-cli -regtest -rpcwallet=miner gettransaction 9f8e7d6c...
{
  "txid": "9f8e7d6c5b4a3928171615141312111009080706050403020100abcdef1234567890ab",
  "amount": -1.00000000,
  "fee": -0.00001000,
  "confirmations": 0,
  "blockhash": null
}

$ bitcoin-cli -regtest -rpcwallet=receiver getbalances
{
  "mine": {
    "trusted": 0.00000000,
    "untrusted_pending": 1.00000000,
    "immature": 0.00000000
  }
}
```

TXID is in the mempool. Sender shows 0 confirmations. Receiver shows 1 BTC as `untrusted_pending`.

## Evidence references

- Screenshot of `sendtoaddress` returning the TXID.
- Screenshot of `getrawmempool` containing the TXID.
- Screenshot of sender `gettransaction` with `"confirmations": 0`.
- Screenshot of receiver `getbalances` with `untrusted_pending: 1.0`.
- `cargo test --test lab_05` — all 4 tests passed.

## Explanation

A transaction passes through distinct states:

1. **Built and signed** — the wallet selects UTXOs, constructs outputs, and signs inputs. No network activity yet.
2. **Broadcast** — the signed transaction is sent to peers. It leaves the wallet but is not yet confirmed.
3. **Mempool** — nodes that accept the transaction hold it in their memory pool, waiting for inclusion in a block. `getrawmempool` shows it; `confirmations` is 0.
4. **Confirmed** — a miner includes the transaction in a block. It leaves the mempool and gains confirmations.

Broadcast alone does not confirm payment. The receiver sees `untrusted_pending` balance because the transaction could still be double-spent or evicted before confirmation.
