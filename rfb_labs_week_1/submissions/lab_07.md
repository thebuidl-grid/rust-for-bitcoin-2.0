# Lab 07 — Confirmation and block membership

## Commands used
Reused the components of Lab 5:
```
btc -rpcwallet=miner gettransaction 77fc470ebfc55a6ed118923afe58621b246c1748b641c7e515aa7e44a48f8ebc

btc getrawmempool

btc getblock <blockhash-from-above>
```


## Terminal output
```
                                                                                                               
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_1]
└─$ btc -rpcwallet=miner gettransaction 77fc470ebfc55a6ed118923afe58621b246c1748b641c7e515aa7e44a48f8ebc
{
  "amount": -1.00000000,
  "fee": -0.00002820,
  "confirmations": 2,
  "blockhash": "1fadaae863695e24b7cdbeb9b30fac2ceb9bd733851012bac17c26e961b2b7e0",
  (....)
}
                                                                                                               
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_1]
└─$ btc getrawmempool
[
]
                                                                                                               
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_1]
└─$ btc getblock 1fadaae863695e24b7cdbeb9b30fac2ceb9bd733851012bac17c26e961b2b7e0
{
  "hash": "1fadaae863695e24b7cdbeb9b30fac2ceb9bd733851012bac17c26e961b2b7e0",
  "confirmations": 2,
  "height": 440,
  "version": 536870912,
  "versionHex": "20000000",
  "merkleroot": "b9ed62866c812e4f48c9b7de8c8d10d5118ac34d2c697c5e33c5be75773cdfbc",
  "time": 1785534756,
(...)}

## Ommitted some of the logs
```

## Evidence references

![ProjectScreenshot](evidence/Lab%207.png)

## Explanation

Before mining, `gettransaction` reported `confirmations: 0` and had no
`blockhash` field at all — the transaction existed only in the mempool,
known to the node but not yet part of the blockchain. After mining a
block, three things changed: `confirmations` became `2` (this block plus
one mined on top of it), a real `blockhash`
(`1fadaae863695e24b7cdbeb9b30fac2ceb9bd733851012bac17c26e961b2b7e0`)
appeared along with `blockheight: 440` and `blockindex: 1`, and the
transaction's `bip125-replaceable` flag flipped from `"yes"` to `"no"` —
once confirmed, it can no longer be replaced via RBF, since it's already
final on-chain. The mempool itself also emptied out (`getrawmempool`
returned `[]`), since a confirmed transaction is removed from the waiting
area — it no longer needs to wait to be mined. Finally, `getblock` on
that hash confirms the transaction is genuinely embedded in the chain: it
appears second in the block's `tx` array (`nTx: 2`, alongside the block's
own coinbase transaction), which is the
