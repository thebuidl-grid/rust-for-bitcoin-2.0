# Lab 07 — Confirmation and block membership

## Commands used
- `bitcoin-cli -regtest -rpcwallet=miner getnewaddress "miner-address-2"` mining address
- `bitcoin-cli -regtest generatetoaddress 1 <address>` backs `mine_one_block`
- `bitcoin-cli -regtest getrawmempool` backs `mempool_is_empty`
- `bitcoin-cli -regtest -rpcwallet=receiver gettransaction <txid>` backs `transaction_confirmations`
- `bitcoin-cli -regtest getblock <blockhash> 1` independently verifies block membership

These compose into `confirm_and_locate_transaction`, which mines a block, checks the
mempool is now empty, reads the wallet's confirmation count, then fetches the actual
block and checks its `tx` array contains the txid proving confirmation independently
of the wallet's own claim.

## Terminal output

$ bitcoin-cli -regtest -rpcwallet=miner getnewaddress "miner-address-2"
bcrt1q2pe5s7dl3wy8mjf26vdx8f78q2sx2jxk4ssy6z

$ bitcoin-cli -regtest generatetoaddress 1 bcrt1q2pe5s7dl3wy8mjf26vdx8f78q2sx2jxk4ssy6z
[
"66c50d2f8574457dcef9ce7effc055f7ba5f2aab9fa4395ae4ae9ba9e602df65"
]

$ bitcoin-cli -regtest getrawmempool
[]

$ bitcoin-cli -regtest -rpcwallet=receiver gettransaction c5c0505f1800e8107d058e5823fb414fdd9c73560cb483f7ef984a31aa82ea54
{
"amount": 1.00000000,
"confirmations": 1,
"blockhash": "66c50d2f8574457dcef9ce7effc055f7ba5f2aab9fa4395ae4ae9ba9e602df65",
"blockheight": 689,
"blockindex": 1,
"blocktime": 1785408427,
"txid": "c5c0505f1800e8107d058e5823fb414fdd9c73560cb483f7ef984a31aa82ea54",

...
}

$ bitcoin-cli -regtest getblock 66c50d2f8574457dcef9ce7effc055f7ba5f2aab9fa4395ae4ae9ba9e602df65 1
{
"hash": "66c50d2f8574457dcef9ce7effc055f7ba5f2aab9fa4395ae4ae9ba9e602df65",
"confirmations": 1,
"height": 689,
"merkleroot": "d2d625deac53d7237dfb71f6a6472b7db1d41f37f9946092f269d69955aa746b",

"nTx": 2,
"previousblockhash": "58e0954b1412ff585b4dc12e99577729cc552498032efb4afccc1a916556faf6",
"tx": [
"9865b495bdbd87d4b7139ef11cdfb997a474205441fcff7c8f69ef6221aa7888",
"c5c0505f1800e8107d058e5823fb414fdd9c73560cb483f7ef984a31aa82ea54"
]
}

## Evidence references (co-authored by Claude)

Captured directly from the local regtest node. Mining one block on top of the
previously-broadcast transaction (Lab 05/06) cleared the mempool to empty and
raised the transaction's confirmation count to 1. The `getblock` call
independently confirms the transaction is genuinely part of block 689's `tx`
array (alongside its coinbase), rather than relying solely on the wallet's
self-reported `confirmations` field.

## Explanation

Before this block was mined, the transaction existed only in the mempool broadcast, valid, but not yet part of the blockchain's permanent history. Mining one block changed several things simultaneously.

First, the mempool emptied out (getrawmempool returned []), because the transaction that had been sitting there pending was now included in a block and is no longer "unconfirmed" it's settled.

Second, gettransaction from the receiver wallet's perspective now reports confirmations: 1 instead of 0, along with a concrete blockhash and blockheight it didn't have before. This is the wallet's own bookkeeping recognizing that the transaction landed in a specific block.

Third and this is the important independent check this lab makes fetching that block directly via getblock and inspecting its tx array shows the transaction's txid actually listed there, right alongside the block's own coinbase transaction. This is a stronger proof than simply trusting the wallet's confirmations field: it's verifying, from the blockchain data itself, that this transaction is genuinely part of a real block that the network has accepted, not just something the wallet believes happened. In a live network, any node could perform this same check independently and reach the same conclusion, which is what makes block membership the actual source of truth for confirmation, rather than any single wallet's internal state.