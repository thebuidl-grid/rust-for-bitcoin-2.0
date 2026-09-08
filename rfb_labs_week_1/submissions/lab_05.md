# Lab 05 — Broadcast and mempool

## Commands used
Here are the commans I used:
```
btc createwallet receiver
btc -rpcwallet=receiver getnewaddress incoming
btc -rpcwallet=miner sendtoaddress <receiver-address> 1
btc getrawmempool                                           # txid is here
btc -rpcwallet=miner gettransaction <txid>

```

## Terminal output

The teminal Output:
```
└─$ btc createwallet receiver       
{
  "name": "receiver"
}

└─$ btc -rpcwallet=receiver getnewaddress incoming
bcrt1q5wwuvau5k76w5tmt5z8msp06x87d5ty53lzafh
                                                                                                             
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_1]
└─$ btc -rpcwallet=miner sendtoaddress bcrt1q5wwuvau5k76w5tmt5z8msp06x87d5ty53lzafh 1
77fc470ebfc55a6ed118923afe58621b246c1748b641c7e515aa7e44a48f8ebc
                                                                                                             
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_1]
└─$ btc getrawmempool                                                                
[
  "77fc470ebfc55a6ed118923afe58621b246c1748b641c7e515aa7e44a48f8ebc"
]
                                                                                                             
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_1]
└─$ btc -rpcwallet=miner gettransaction 77fc470ebfc55a6ed118923afe58621b246c1748b641c7e515aa7e44a48f8ebc
{
  "amount": -1.00000000,
  "fee": -0.00002820,
  "confirmations": 0,
  "trusted": true,
  "txid": "77fc470ebfc55a6ed118923afe58621b246c1748b641c7e515aa7e44a48f8ebc",
  "wtxid": "93e8576e91766dfd97433c1169e1bc1dc5348eb2905734749f18feda39be9b33",
  "walletconflicts": [
  ],
  "mempoolconflicts": [
  ],
  "time": 1785533627,
  "timereceived": 1785533627,
  "bip125-replaceable": "yes",
  "details": [
    {
      "address": "bcrt1q5wwuvau5k76w5tmt5z8msp06x87d5ty53lzafh",
      "category": "send",
      "amount": -1.00000000,
      "vout": 1,
      "fee": -0.00002820,
      "abandoned": false
    }
  ],
  "hex": "020000000001017becafa1467e8a4f5693f6683ed29c1a9b7cc5aa79c5d7dc26cc37ddb28c77390000000000fdffffff027c908b44000000001600149f23f1230f875720204208618156ccf28d48203300e1f50500000000160014a39dc67794b7b4ea2f6ba08fb805fa31fcda2c940247304402207257bf173798a1083affa31583f1d8ddb362fa6ea2778676695e2a6f6daed5f102203f3ae5772cc3f193f9db4360e9f1e513b8ba45a908fa5c7835a1744c8ac483a50121027364fc83502ceb92c9201132a7aff61f512941f28ecf2d9497a578ade91e69d4b7010000",
  "lastprocessedblock": {
    "hash": "2dc824c56d8817746a0a71e3a39101755c7696ef087d673698c915835e63858e",
    "height": 439
  }
}

```

## Evidence references

![ProjectScreenshot](evidence/Lab%205.png)

## Explanation

A Bitcoin transaction passes through several distinct states before it's
considered final, and each state means something different for whether
the funds involved can be trusted or relied upon:

**Signed** — the transaction has been constructed and cryptographically
signed by the wallet (proving ownership of the inputs being spent), but
it exists only locally — it hasn't been sent anywhere yet. At this point
it's not visible to the network or the receiving party at all.

**Broadcast** — the signed transaction has been sent out to the network.
In this lab, `sendtoaddress` handles signing and broadcasting in one
step, immediately returning a `txid`. Once broadcast, other nodes on the
network (and the sending node itself) become aware the transaction
exists.

**Mempool (unconfirmed)** — after broadcast, the transaction sits in
nodes' memory pools ("mempool") — a waiting area of transactions that are
valid and known to the network but not yet included in any block. This
is exactly what `getrawmempool` showed: the txid present, no block
involved yet. At this stage `gettransaction` reports `confirmations: 0`,
and — critically — the funds are *not yet safe to treat as final*. The
sender's wallet already reflects the payment as sent (a negative
`amount` in `gettransaction`, since the outgoing funds are committed from
the sender's perspective as soon as it's broadcast), but the receiver's
wallet only shows the incoming funds as `untrusted_pending`, not
`trusted` — because an unconfirmed transaction could still theoretically
be replaced, double-spent, or simply never get mined at all if fees are
too low.

**Confirmed** — once a miner includes the transaction in a mined block,
it becomes confirmed. `gettransaction` starts reporting a real
`blockhash` and a `confirmations` count of at least 1, which increases as
more blocks are mined on top. Once confirmed, the receiving wallet's
balance for those funds moves from `untrusted_pending` into `trusted`,
and (per the coinbase maturity rule for mined rewards specifically) can
eventually be spent onward.

This progression — signed → broadcast → mempool → confirmed — reflects
increasing certainty that the transaction is permanently part of the
blockchain's history. Each stage is directly observable through Bitcoin
Core's RPC interface, which is what makes it possible to build tooling
(like this lab's functions) that can distinguish "money has been sent"
from "money has actually, irreversibly arrived."
