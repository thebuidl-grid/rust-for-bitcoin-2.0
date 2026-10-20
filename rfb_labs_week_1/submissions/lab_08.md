# Lab 08 — Block security

## Commands used

cargo test --test lab_08
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getblockheader 30152aa933a63b1139659752d9f8dabed27221d9d27104fbb9ed37764944c34c
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=receiver gettransaction 608468411918a587f203ec07024d44cc5a227be5f1f253207c809b6239f39272
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner getnewaddress mining8
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie generatetoaddress 5 "$MINER_ADDR"
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=receiver gettransaction 608468411918a587f203ec07024d44cc5a227be5f1f253207c809b6239f39272

## Terminal output

- Block header (`getblockheader 30152aa...`):
	- hash: `30152aa933a63b1139659752d9f8dabed27221d9d27104fbb9ed37764944c34c`
	- previousblockhash: `15da3da46c3d5d5ad60177083cd1f2b2e517607c4d8e0a49e8506521bcf6a49a`
	- merkleroot: `f7dec9c740029679794e9238c28987f962eaaaca026757319d58e20c3a83d680`
	- bits: `207fffff`
	- nonce: `0`
	- confirmations: `6`
- Receiver transaction (`gettransaction 608468...`):
	- txid: `608468411918a587f203ec07024d44cc5a227be5f1f253207c809b6239f39272`
	- amount: `1.00000000`
	- blockhash: `30152aa933a63b1139659752d9f8dabed27221d9d27104fbb9ed37764944c34c`
	- confirmations moved from `1` to `6` after mining 5 additional blocks

## Evidence references

- `submissions/evidence/lab8.png`
- `submissions/evidence/lab8-2.png`

## Explanation

The block header commits to both chain history and all included transactions. `previousblockhash` links this block to its parent, so changing any earlier block would break this link unless all descendant blocks are rebuilt.

The `merkleroot` commits to the full transaction set in the block. If any transaction byte changed, the Merkle root would change and the block hash would no longer match.

`bits` encodes the proof-of-work target and `nonce` is part of the solved header fields used by miners to satisfy that target. Together, these show the block was mined under consensus rules.

Confirmation depth increased from 1 to 6 after mining 5 more blocks on top of the transaction's block. More confirmations means more accumulated work would need to be replaced to reverse this payment, so the transaction is increasingly secure against reorg risk.
