# Lab 07 — Confirmation and block membership

## Commands used

cargo test --test lab_07
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner getnewaddress mining7
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie generatetoaddress 1 "$MINER_ADDR"
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getrawmempool
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=receiver gettransaction "$TXID"
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getblock 30152aa933a63b1139659752d9f8dabed27221d9d27104fbb9ed37764944c34c 1

## Terminal output

- `getrawmempool` output: `[]` (empty mempool after mining)
- Receiver transaction summary (`listtransactions` / `gettransaction`):
	- txid: `608468411918a587f203ec07024d44cc5a227be5f1f253207c809b6239f39272`
	- amount: `1.00000000`
	- confirmations: `1`
	- blockhash: `30152aa933a63b1139659752d9f8dabed27221d9d27104fbb9ed37764944c34c`
- `getblock ... 1` for that block hash:
	- height: `205`
	- `tx` includes:
		- `9d38bd6291499dcbcddf6e80f0adc519777b1649086234e36a17c65d3f370bd9` (coinbase)
		- `608468411918a587f203ec07024d44cc5a227be5f1f253207c809b6239f39272` (payment tx)

## Evidence references

- `submissions/evidence/lab7.png` (combined terminal capture for mempool empty, receiver transaction confirmation, and block tx membership)

## Explanation

Before mining, the transaction existed as an unconfirmed mempool entry. After mining exactly one block, two observable things changed: it disappeared from the mempool and gained one confirmation in the receiver wallet (`confirmations = 1`) with a concrete containing `blockhash`.

The transaction itself (its txid and serialized structure) did not change. What changed is its position in chain history: it was included in a block, and that block's `tx` list now contains the transaction id. This is the difference between broadcast and confirmation.
