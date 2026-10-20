# Lab 04 — UTXOs and outpoints

## Commands used
cargo test --test lab_04
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner listunspent
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner getbalances

## Terminal output

Selected spendable UTXO from `listunspent`:

- txid: `b3aae9043edfeb56e0e4a0e0b9ffbb9d509ed7a785041791aa15205fa0cf37d1`
- vout: `0`
- amount: `50.00000000`
- confirmations: `101`
- address: `bcrt1qx03wsfr4269heul7f28yw5kkn6zj6exp8xhk40`
- scriptPubKey: `001433e2e82475568b7cf3fe4a8e4752d69e852d64c1`
- spendable: `true`

Constructed outpoint:

- `b3aae9043edfeb56e0e4a0e0b9ffbb9d509ed7a785041791aa15205fa0cf37d1:0`

Spendable-sum reconciliation with wallet balance:

- sum of spendable UTXOs: `50.00000000 BTC`
- wallet trusted balance (`getbalances.mine.trusted`): `50.00000000 BTC`
- reconciliation: spendable UTXO sum matches trusted wallet balance.

## Evidence references

- `submissions/evidence/lab4-listunspent.png` (selected spendable UTXO: `txid`, `vout`, amount, confirmations, and `scriptPubKey`)
- `submissions/evidence/lab4-balances.png` (`getbalances` output used to reconcile trusted wallet balance with spendable UTXO sum)

## Explanation

A UTXO is a specific spendable transaction output, and an outpoint (`txid:vout`) is the unique coordinate that identifies it. Wallet balance is not an account row; it is an aggregate computed from many UTXOs controlled by the wallet. In this lab, I identified one spendable UTXO, built its outpoint, then compared the sum of spendable UTXOs with the wallet trusted balance to show they reconcile.
