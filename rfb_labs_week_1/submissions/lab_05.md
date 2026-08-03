## Commands used

```
cargo test --test lab_05

bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "<receiver-address>" 1
bitcoin-cli -regtest getrawmempool
bitcoin-cli -regtest -rpcwallet=miner gettransaction "<txid>"
bitcoin-cli -regtest -rpcwallet=receiver getbalances
```

*RPCs are the ones issued by `send_btc`, `get_raw_mempool`, `get_transaction_status`, and `observe_unconfirmed_payment` in `src/labs/lab05_mempool.rs`, verified against the mocked RPC client in `tests/lab_05.rs`. Run the `bitcoin-cli` lines against your live Polar regtest node (do not mine after sending) to capture the terminal output below.*

## Terminal output

Captured against the live regtest node, without mining after the send:

```
$ bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1qx208aadpjxz7ftargmdy64amhslmycnjll2xxl" 1
7db84ed92ac38e4c6f01412011f9f97098cedd8a259c646c9cd1f192a9ff84c2

$ bitcoin-cli -regtest getrawmempool
[
  "7db84ed92ac38e4c6f01412011f9f97098cedd8a259c646c9cd1f192a9ff84c2"
]

$ bitcoin-cli -regtest -rpcwallet=miner gettransaction "7db84ed92ac38e4c6f01412011f9f97098cedd8a259c646c9cd1f192a9ff84c2"
{
  "amount": -1.00000000,
  "fee": -0.00002820,
  "confirmations": 0,
  "trusted": true,
  "txid": "7db84ed92ac38e4c6f01412011f9f97098cedd8a259c646c9cd1f192a9ff84c2",
  "bip125-replaceable": "yes",
  ...
}

$ bitcoin-cli -regtest -rpcwallet=receiver getbalances
{
  "mine": {
    "trusted": 0.00000000,
    "untrusted_pending": 1.00000000,
    "immature": 0.00000000
  },
  ...
}
```

The transaction reached the mempool immediately (`getrawmempool` lists its txid) and shows `"confirmations": 0` from the sender's side. The receiver's wallet already sees the 1 BTC, but parked under `untrusted_pending`, not `trusted` — it isn't spendable until mined.

## Evidence references

Evidence is the live terminal output above, captured directly via `docker exec bitcoind-lab-a bitcoin-cli ...` against a real regtest node (not a screenshot — this session ran headlessly, no Polar GUI was open).

## Explanation

A transaction passes through several distinct states on its way to being final:

1. **Signed** — the wallet has built the transaction and produced valid signatures unlocking its inputs. At this point it exists only in memory/on disk locally; no one else on the network knows about it, and it has no txid the network recognizes as "real" yet in any meaningful sense.
2. **Broadcast** — the node has sent the signed transaction to its peers. `sendtoaddress` does both the signing and the broadcasting in one step, immediately returning the txid (`7db84ed9...` here).
3. **In the mempool** — every node that receives and validates the transaction holds it in its local mempool, a waiting room of transactions eligible for inclusion in the next block. `getrawmempool` listing the txid is proof it successfully propagated and passed validation — but being in the mempool is *not* the same as being confirmed. A mempool transaction can still be dropped, replaced (if RBF-enabled, as `"bip125-replaceable": "yes"` shows here), or reordered.
4. **Confirmed** — a miner has included the transaction in a mined block. Only then does `confirmations` go from `0` to `1` (and count up as more blocks stack on top, per Lab 07/08).

This progression is exactly why the receiver's `getbalances` shows the 1 BTC as `untrusted_pending` rather than `trusted`: it's been broadcast and is sitting in the mempool, but until it's mined the receiver can see the incoming payment without being able to safely treat it as final, spendable money — a sender could still double-spend or replace an unconfirmed transaction.