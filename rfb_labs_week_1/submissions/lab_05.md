# Lab 05 — Broadcast and mempool

## Commands used

```bash
cargo run -- lab05
```

```bash
bitcoin-cli ... -rpcwallet=miner sendtoaddress bcrt1qga5wdzs456gvrk7kzh07lxxm5lxjarslkxm3m4 1
bitcoin-cli ... getrawmempool
bitcoin-cli ... -rpcwallet=miner    gettransaction f4ddd9cb43e8ec40c6ca4d34a2ba8407b285f5daefd93d1e73a3621ed82f532d
bitcoin-cli ... -rpcwallet=receiver getbalances
```

No block is mined anywhere in this lab. That is the whole point of it.

## Terminal output

```text
$ bitcoin-cli ... -rpcwallet=miner sendtoaddress bcrt1qga5wdzs...m3m4 1
  f4ddd9cb43e8ec40c6ca4d34a2ba8407b285f5daefd93d1e73a3621ed82f532d

$ bitcoin-cli ... getrawmempool
  [
    "f4ddd9cb43e8ec40c6ca4d34a2ba8407b285f5daefd93d1e73a3621ed82f532d"
  ]

$ bitcoin-cli ... -rpcwallet=miner gettransaction f4ddd9cb...532d
  {
    "amount": -1.00000000,
    "fee": -0.00002820,
    "confirmations": 0,
    "trusted": true,
    "txid": "f4ddd9cb43e8ec40c6ca4d34a2ba8407b285f5daefd93d1e73a3621ed82f532d",
    "bip125-replaceable": "yes",
    "details": [ { "address": "bcrt1qga5wdzs...m3m4", "category": "send", "amount": -1.00000000, "vout": 0 } ]
  }

$ bitcoin-cli ... -rpcwallet=receiver getbalances
  { "mine": { "trusted": 0.00000000, "untrusted_pending": 1.00000000, "immature": 0.00000000 } }
```

Assembled observation:

```json
{
  "txid": "f4ddd9cb43e8ec40c6ca4d34a2ba8407b285f5daefd93d1e73a3621ed82f532d",
  "mempool_contains_tx": true,
  "sender_status": { "confirmations": 0, "amount": -1.0, "fee": -0.0000282, "block_hash": null },
  "receiver_balance": { "trusted": 0.0, "untrusted_pending": 1.0, "immature": 0.0 }
}
```

All four required facts are here: the TXID is in `getrawmempool`; the sender reports
`confirmations: 0`; the receiver sees `untrusted_pending: 1.0` and `trusted: 0.0`; and
`block_hash` is `null` because no block contains it.

## Evidence references

Full run log at `evidence/week1-labs-01-09.log`, lines 386-457, including the complete
`gettransaction` output with the raw transaction hex.

## Explanation

The four states are distinct, and this lab sits deliberately in the middle two.

**Built and signed.** The wallet picks input UTXOs, builds outputs, and signs. At this
point the transaction exists only in my node's memory — nobody else has heard of it and
nothing about the network has changed. It could be discarded with no trace.

**Broadcast.** The transaction is relayed to peers. This is a networking event, not a
consensus one. It says nothing about whether the transaction will ever be included.

**In the mempool.** Each node independently validates the transaction — signatures check
out, the inputs exist and are unspent, the fee clears the node's minimum — and holds it
in memory as a candidate for a future block. My TXID appearing in `getrawmempool` proves
my node accepted it. But a mempool is per-node and purely local. Another node might have
a different mempool, might have evicted it, or might never have seen it. There is no
single global mempool.

**Confirmed.** A miner includes it in a block, that block satisfies proof of work, and
nodes accept it. Only now is it part of the agreed history.

`confirmations: 0` names exactly this gap. The transaction is valid and known, but zero
blocks commit to it, so nothing stops it being replaced. My own output flags
`"bip125-replaceable": "yes"` — the sender can broadcast a conflicting version with a
higher fee and replace it outright.

That is what the receiver's balance is expressing. The 1 BTC shows as
`untrusted_pending`, not `trusted`, because the wallet knows about the payment but is
declining to treat it as settled money. "Untrusted" is not a comment on my honesty as the
sender; it is the wallet correctly refusing to count value that no block has committed to
yet. Accepting a zero-confirmation payment as final is exactly how double-spend attacks
succeed, and this balance split is Bitcoin Core telling me not to.
