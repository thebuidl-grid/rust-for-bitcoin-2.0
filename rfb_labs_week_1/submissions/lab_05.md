# Lab 05 — Broadcast and mempool

## Commands used

cargo test --test lab_05
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=receiver getnewaddress classmate
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner sendtoaddress "$RECEIVER_ADDR" 1
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getrawmempool
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner gettransaction "$TXID"
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=receiver getbalances

## Terminal output

- Receiver address generated:
	- `bcrt1q8zmquj362hyyqu9uawvk63hj4e937jyd3fav6p`
- Broadcast TXID:
	- `608468411918a587f203ec07024d44cc5a227be5f1f253207c809b6239f39272`
- `getrawmempool` contained that TXID immediately after broadcast.
- Sender wallet view (`gettransaction`):
	- `confirmations: 0`
	- `fee: -0.00002820`
	- `bip125-replaceable: yes`
- Receiver wallet balances (`getbalances`):
	- `trusted: 0.00000000`
	- `untrusted_pending: 1.00000000`
	- `immature: 0.00000000`

## Evidence references

- `submissions/evidence/lab5.png` (terminal evidence for transaction broadcast and mempool checks)
- `submissions/evidence/lab5-balances.png` (receiver wallet balance evidence showing pending/trusted state)

## Explanation

In this lab, the payment moves through two distinct states before confirmation:

- Built and signed: `sendtoaddress` creates and signs a transaction in the sender wallet, then returns a TXID.
- Broadcast and mempool: the node accepts and relays the transaction to its local mempool (`getrawmempool` contains the TXID), but it still has zero confirmations in `gettransaction`.

The receiver can already see value, but as `untrusted_pending` in `getbalances`, not as trusted spendable balance. A transaction is only confirmed after miners include it in a block. So broadcast is network propagation, while confirmation is chain inclusion.
