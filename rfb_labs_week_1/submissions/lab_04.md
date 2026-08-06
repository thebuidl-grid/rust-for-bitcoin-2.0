# Lab 04 — UTXOs and outpoints

## Commands used

```bash
cargo test --test lab_04
bitcoin-cli -regtest -rpcwallet=miner listunspent
bitcoin-cli -regtest -rpcwallet=miner getbalance
```

## Terminal output

```text
Selected TXID: [PASTE ACTUAL TXID]
Selected vout: [PASTE ACTUAL VOUT]
Outpoint: [PASTE TXID:VOUT]
Amount: [PASTE ACTUAL BTC AMOUNT]
Confirmations: [PASTE ACTUAL CONFIRMATIONS]
Address: [PASTE ACTUAL ADDRESS]
scriptPubKey: [PASTE ACTUAL SCRIPT]
Spendable: [PASTE ACTUAL VALUE]
Sum of spendable UTXOs: [PASTE CALCULATED TOTAL]
Bitcoin Core wallet balance: [PASTE GETBALANCE RESULT]
Rust tests: [PASTE PASSING TEST SUMMARY]
```

## Evidence references

- [`listunspent` output and wallet balance reconciliation](evidence/lab_04.png)

## Explanation

A UTXO is a transaction output that has not yet been spent. An outpoint identifies one specific output using its transaction ID and output index in the form `txid:vout`. Bitcoin does not maintain an account-style balance entry. A wallet finds outputs whose locking conditions it can satisfy and calculates its available balance by summing the spendable UTXOs it controls.
