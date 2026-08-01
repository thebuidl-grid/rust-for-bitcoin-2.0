# Lab 04 — UTXOs and outpoints

## Commands used

<!-- TODO: Record the commands used to inspect and calculate wallet UTXOs. -->
```bash
bitcoin-cli -rpcwallet=miner listunspent  # List all UTXOs


bitcoin-cli -rpcwallet=miner getbalances # Get wallet balances


bitcoin-cli getrawtransaction <txid> true # Get transaction details
```

## Terminal output

<!-- TODO: Include txid, vout, amount, confirmations, script, and spendable state. -->
```bash

```

## Evidence references

<!-- TODO: Link screenshots or describe the attached evidence. -->
The first screenshot show list of utxo of miner wallets


![alt text](../screenshots/lab004_list_miner_unspent_utsx.png)

The second screensht show transaction details of one of the UTXO from the transaction id


![alt text](<../screenshots/lab004_get_transaction_details_of_a utxo.png>)

The third screenshot show the test result of lba004 implementation


![alt text](../screenshots/lab004_test.png)
## Explanation

<!-- TODO: Explain outpoints, UTXOs, and why a wallet balance is their sum. -->

Outpoint
A pointer to a specific output — identified by (txid, index). It says "output #N of transaction X."

UTXO (Unspent Transaction Output)
An output that hasn't been spent yet. Each UTXO has a value (in BTC/sats) and a locking condition (who can spend it). An outpoint is how you reference a UTXO; the UTXO is the actual coin sitting there.

Why balance = sum of UTXOs
Bitcoin has no account balances. Your wallet doesn't store "you have 2 BTC" — it just tracks which UTXOs are spendable by your keys. Balance is just the total value of all UTXOs the wallet controls:
