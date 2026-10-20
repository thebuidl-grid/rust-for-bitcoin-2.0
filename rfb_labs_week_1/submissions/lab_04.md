# Lab 04 — UTXOs and outpoints

## Commands used

TODO: Record the commands used to inspect and calculate wallet UTXOs.

`cargo test --test lab_04` ran the Lab 04 test 

`bitcoin-cli -regtest -rpcwallet=test listunspent ` lists all unspent transaction outputs (UTXOs) that belong to the specified wallet and are available to spend

## Terminal output

TODO: Include txid, vout, amount, confirmations, script, and spendable state.
<img width="1130" height="274" alt="Screenshot 2026-08-02 at 00 42 38" src="https://github.com/user-attachments/assets/dfb1e8e9-4a97-46cf-a155-cd47c34d4e49" />


## Evidence references

TODO: Link screenshots or describe the attached evidence.

`listunspent` shows the **UTXOs (Unspent Transaction Outputs)** that belong to that wallet and are currently spendable. its like saying, “Show me all the coins this wallet owns that have not yet been spent.” It contains fields like txid(The transaction that created the output), address(The wallet address that received the coins), and confirmation (the number of confirmations).


## Explanation

TODO: Explain outpoints, UTXOs, and why a wallet balance is their sum.

Outpoint: the unique identifier of a UTXO, written as (txid, vout), where txid is the transaction hash and vout is the output index.

UTXO (Unspent Transaction Output): a specific coin output created by a transaction that has not yet been spent.

A wallet owns a set of UTXOs; its balance is simply the sum of the amounts of all spendable UTXOs it controls. When one UTXO is spent, it disappears, and new UTXOs (payment + change) are created.
