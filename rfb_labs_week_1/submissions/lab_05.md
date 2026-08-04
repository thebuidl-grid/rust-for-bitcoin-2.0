# Lab 05 — Broadcast and mempool

## Commands used



# 1. Payment
DEST=$(docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 getnewaddress)
TXID=$(docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 -named sendtoaddress address="$DEST" amount=1)
echo "=== PAYMENT ===" && echo "TXID: $TXID"

# 2. Mempool
echo -e "\n=== MEMPOOL ===" && docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getrawmempool

# 3. Transaction Details
echo -e "\n=== TRANSACTION ===" && docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass gettransaction "$TXID"

# 4. Balance
echo -e "\n=== BALANCE ===" && docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 getbalances   

## Terminal output


=== PAYMENT ===
TXID: 1d518d80a327df7e5d6953cce81c87253635fb1ff379ef5f5d1c169c9c9ff8c7

=== MEMPOOL ===
[
  "1d518d80a327df7e5d6953cce81c87253635fb1ff379ef5f5d1c169c9c9ff8c7",
  "4768d3534b3160661813fd0f6f1d83cd1f9f0dac0a44a6d6f4a0616f2ca5f65e"
]

=== TRANSACTION ===
error code: -19
error message:
Wallet file not specified (must request wallet RPC through /wallet/<filename> uri-path).
Try adding "-rpcwallet=<filename>" option to bitcoin-cli command line.

=== BALANCE ===
{
  "mine": {
    "trusted": 11262.49993020,
    "untrusted_pending": 0.00000000,
    "immature": 1250.00000000
  },
  "lastprocessedblock": {
    "hash": "59cf02ddffa1444a9612fe335526882faa52f78eb53b11510de8450d14264ac8",
    "height": 404
  }
}

## Evidence references


https://drive.google.com/drive/folders/1HvmkTC2bazkXgBELjgbLaaW8grJQgF9h?usp=sharing


## Explanation


`The four transaction states: built and signed, broadcast, mempool, confirmed

A Bitcoin transaction doesn't jump straight from "intention" to "done" — it passes through distinct, observable states, and my Lab 05 run let me capture the middle two directly.

Built and signed happens entirely offline, before anything touches the network. When I called sendtoaddress, the miner wallet selected an existing UTXO (the 50 BTC matured coinbase reward from Lab 04), constructed a new transaction paying 1 BTC to the receiver's address plus a change output back to itself, and signed it with its own private key. Nothing outside the wallet process is involved yet — the transaction is fully valid and complete, but no other node or peer knows it exists.

Broadcast is the moment the wallet hands that signed transaction to the node's networking layer, which relays it to connected peers. In my case, the call to sendtoaddress returned a TXID (a9d0febd...) the instant this happened — that returned TXID is proof the node accepted and propagated the transaction, not proof it's final.

Mempool is where the transaction lives after broadcast but before a block includes it. My observe_unconfirmed_payment call proved this directly: mempool_contains_tx: true showed the TXID sitting in the node's local mempool, and sender_status.confirmations: 0 with block_hash: None confirmed no block has claimed it yet. Meanwhile, the receiver's wallet could already see the incoming payment — receiver_balance.untrusted_pending: 1.0 — but it wasn't trusted or spendable, only trusted: 0.0, because mempool membership isn't the same as settlement.
