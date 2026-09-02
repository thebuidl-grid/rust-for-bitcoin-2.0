# Lab 07 — Confirmation and block membership

## Commands used

```bash
cargo run -- lab07
```

```bash
bitcoin-cli ... generatetoaddress 1 bcrt1qfsw0fvcdjruj7d746sxqy0nnnpptcvsyslhx0q
bitcoin-cli ... getrawmempool
bitcoin-cli ... -rpcwallet=receiver gettransaction f4ddd9cb43e8ec40c6ca4d34a2ba8407b285f5daefd93d1e73a3621ed82f532d
bitcoin-cli ... getblock 437082f60d095e6233d0cef531f9b5ae193ec02aaf818d9126821dc46585fe32 1
```

I check membership from two independent directions: the wallet's claim about which block
holds the transaction, and the block's own `tx` list. One `gettransaction` call supplies
both the confirmation count and the block hash.

## Terminal output

```text
$ bitcoin-cli ... generatetoaddress 1 bcrt1qfsw0f...
  [ "437082f60d095e6233d0cef531f9b5ae193ec02aaf818d9126821dc46585fe32" ]

$ bitcoin-cli ... getrawmempool
  [
  ]

$ bitcoin-cli ... -rpcwallet=receiver gettransaction f4ddd9cb...532d
  {
    "amount": 1.00000000,
    "confirmations": 1,
    "blockhash": "437082f60d095e6233d0cef531f9b5ae193ec02aaf818d9126821dc46585fe32",
    "blockheight": 103,
    "blockindex": 1,
    "bip125-replaceable": "no",
    "details": [ { "category": "receive", "amount": 1.00000000, "label": "classmate" } ]
  }

$ bitcoin-cli ... getblock 437082f6...fe32 1
  {
    "hash": "437082f60d095e6233d0cef531f9b5ae193ec02aaf818d9126821dc46585fe32",
    "height": 103,
    "nTx": 2,
    "merkleroot": "798e557fb4814cce9f65a88174925e38f0687172ffcde76121641d0a75fd30f5",
    "tx": [
      "02fe6668e206f80cc6eb3a5a9fd3dffd868cbeaa78d47a62ca8dddba7963ab2f",
      "f4ddd9cb43e8ec40c6ca4d34a2ba8407b285f5daefd93d1e73a3621ed82f532d"
    ]
  }
```

Report, plus the receiver's balance afterwards:

```json
{
  "txid": "f4ddd9cb43e8ec40c6ca4d34a2ba8407b285f5daefd93d1e73a3621ed82f532d",
  "block_hash": "437082f60d095e6233d0cef531f9b5ae193ec02aaf818d9126821dc46585fe32",
  "confirmations": 1,
  "mempool_is_empty": true,
  "transaction_is_in_block": true
}

receiver balances: { "trusted": 1.0, "untrusted_pending": 0.0, "immature": 0.0 }
```

The mempool is empty, the transaction has one confirmation, Bitcoin Core names block
`437082f6…fe32` at height 103, and that block's `tx` array contains my TXID at index 1
(index 0 is the coinbase). The receiver's 1 BTC moved from `untrusted_pending` to
`trusted`.

## Evidence references

Full run log at `evidence/week1-labs-01-09.log`, lines 625-724. Comparing the `hex` field
at line 665 with the same field in Lab 05 (line 420) shows the two are byte-identical.

## Explanation

Mining did **not** change the transaction. I checked this rather than assuming it: the
`hex` field in `gettransaction` is byte-for-byte identical before and after confirmation,
and the `wtxid` is the same `1869cda352d0…8caf` in both. That has to be true — the txid
*is* a hash of the serialized transaction, so any change to the bytes would produce a
different txid and it would no longer be the same transaction at all. The inputs, the
outputs, the signatures and the fee are all exactly what they were when I broadcast it.

What changed is its **position in the agreed history**. Before, it was a candidate sitting
in one node's mempool, valid but committed to by nothing. Now block `437082f6…fe32`
includes it, and that block's Merkle root `798e557f…30f5` is a hash commitment over its
two transactions. Altering or removing my transaction would change the Merkle root, which
would change the block header, which would invalidate the proof of work already spent on
it. That is the difference between "known" and "settled".

Three consequences visible in the output. The mempool emptied because a mempool holds
transactions *awaiting* inclusion, and mine no longer qualifies. `bip125-replaceable`
flipped from `yes` to `no` — fee-bumping a confirmed transaction is no longer possible,
because replacing it would now require redoing the block. And the receiver's wallet moved
the 1 BTC from `untrusted_pending` to `trusted`, since there is finally a block committing
to the payment.

One confirmation is real but thin. Block 103 is the current tip, so a competing branch of
the same length could still displace it. That is what the next lab measures.
