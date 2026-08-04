# Lab 05 — Broadcast and mempool

## Commands used
- `bitcoin-cli -regtest -rpcwallet=miner sendtoaddress <receiver-address> 1` backs `send_btc`
- `bitcoin-cli -regtest getrawmempool` - backs `get_raw_mempool`
- `bitcoin-cli -regtest -rpcwallet=miner gettransaction <txid>` backs `get_transaction_status`
- `bitcoin-cli -regtest -rpcwallet=receiver getbalances` reads the receiver's pending balance

These compose into `observe_unconfirmed_payment`, which sends a payment, checks the
mempool, inspects the sender's transaction status, and reads the receiver's wallet
balance - all without mining a confirming block in between.

## Terminal output

$ bitcoin-cli -regtest -rpcwallet=miner sendtoaddress bcrt1q82tmmnmf77qymd5kg6k7ly3k4a45pfp8l9xxlr 1
c5c0505f1800e8107d058e5823fb414fdd9c73560cb483f7ef984a31aa82ea54

$ bitcoin-cli -regtest getrawmempool
[
"c5c0505f1800e8107d058e5823fb414fdd9c73560cb483f7ef984a31aa82ea54"
]

$ bitcoin-cli -regtest -rpcwallet=miner gettransaction c5c0505f1800e8107d058e5823fb414fdd9c73560cb483f7ef984a31aa82ea54
{"amount": -1.00000000,
"fee": -0.00001410,
"confirmations": 0,
"trusted": true,
"txid": "c5c0505f1800e8107d058e5823fb414fdd9c73560cb483f7ef984a31aa82ea54",
"wtxid": "87a377abe51f1e8634eeb54348c08fa1afff49126124bc2bd15e8b3473528282",
"walletconflicts": [],
"time": 1785313089,
"timereceived": 1785313089,
"bip125-replaceable": "yes",
"details": [{
"address": "bcrt1q82tmmnmf77qymd5kg6k7ly3k4a45pfp8l9xxlr",
"category": "send",
"amount": -1.00000000,
"vout": 1,
"fee": -0.00001410,
"abandoned": false
}
],
"hex": "02000000000101c8e18cbf8de09f04d7f102f08cddd0a44f25f8d755d78f25698e26caaf39602a0000000000fdffffff02bed74a1f00000000160014c7a02e202d91d4bb08ff71da3ce32dd4879739ce00e1f505000000001600143a97bdcf69f7804db69646adef9236af6b40a4270247304402206df933b8e86deb656fc406cd109bc811b1e620bd7e9eeda080a56075e894923d02202e14869ec3e126222e4267059502b285a2c1e3270296112293311422a86ad30601210237e46bb8f87939a87634f2bb2a7cc626a7bbb653a0f1cf2d2cf43c6022c05c66b0020000"
}
$ bitcoin-cli -regtest -rpcwallet=receiver getbalances
{
"mine": {
"trusted": 0.00000000,
"untrusted_pending": 1.00000000,
"immature": 0.00000000
}
}

## Evidence references
Captured directly from the local regtest node. The `miner` wallet's 6.25 BTC
spendable UTXO from Lab 03/04 was used to fund a 1 BTC payment to the `receiver`
wallet. All three checks (mempool, gettransaction, receiver balance) were run
before mining any new block, proving the transaction was broadcast but not yet
confirmed.

## Explanation  (co-authored by Claude)

A Bitcoin transaction goes through several distinct states before it's truly final, and this lab captures the gap between two of them: broadcast and confirmed. When sendtoaddress is called, the wallet first constructs the transaction, signs it with the private key controlling the input UTXO, and then broadcasts it to the network at that point it's a valid, fully-signed transaction, but it isn't part of any block yet.

Once broadcast, the transaction lands in the mempool: a pool that every full node maintains locally, holding all transactions that node considers valid and is willing to relay, but which haven't been mined into a block. getrawmempool reflects exactly this the txid c5c0505f... appearing there confirms the node has accepted and is holding the transaction, waiting for a miner to include it. Because every node keeps its own mempool independently, in the mempool only means "known and pending to this particular node, not "guaranteed to be confirmed.

The transaction's own status still reports confirmations: 0 from gettransaction, since it hasn't been included in any block confirmation count only starts incrementing once a block containing the transaction is mined and 0 means "seen, but not yet settled." From the receiving side, this state is reflected in the untrusted_pending balance: receiver's getbalances shows the incoming 1 BTC there rather than in trusted, because Bitcoin Core doesn't treat unconfirmed incoming funds as fully reliable the transaction is bip125-replaceable: "yes" in this evidence, meaning it could technically still be replaced (via RBF) or fail to ever confirm, so counting on it before confirmation would be premature.

Only once the transaction is actually included in a mined block does its state change again moving from "broadcast and pending in the mempool" to "confirmed," at which point it leaves the mempool entirely and its funds become trusted on the receiving side. This progression (signed -> broadcast -> mempool -> confirmed) is the core lifecycle every Bitcoin transaction passes through, and this lab specifically freezes the process at the "broadcast but unconfirmed" stage to make that intermediate state visible.