# Lab 05 — Broadcast and mempool

## Commands used

cargo test --test lab_05
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=receiver getnewaddress classmate
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner sendtoaddress <receiver_address> 1
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getrawmempool
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner gettransaction <txid>
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=receiver getbalances

## Terminal output

TODO: Show the TXID, zero confirmations, mempool entry, and pending balance.

## Evidence references

TODO: Link screenshots or describe the attached evidence.

## Explanation

In this lab, the payment moves through two distinct states before confirmation:

- Built and signed: `sendtoaddress` creates and signs a transaction in the sender wallet, then returns a TXID.
- Broadcast and mempool: the node accepts and relays the transaction to its local mempool (`getrawmempool` contains the TXID), but it still has zero confirmations in `gettransaction`.

The receiver can already see value, but as `untrusted_pending` in `getbalances`, not as trusted spendable balance. A transaction is only confirmed after miners include it in a block. So broadcast is network propagation, while confirmation is chain inclusion.
