# Lab 04 — UTXOs and outpoints

## Commands used

```bash
cargo test --test lab_04
bitcoin-cli -regtest -rpcwallet=miner listunspent
bitcoin-cli -regtest -rpcwallet=miner getbalances
```

## Terminal output

The selected spendable output included a `txid`, `vout`, BTC amount, confirmation count, address, `scriptPubKey`, and `spendable=true`. Its outpoint was constructed as the pair `txid:vout`. Summing only spendable UTXOs matched the wallet's trusted spendable balance.

## Evidence references

Evidence is the Lab 04 test run and the `listunspent` output for the miner wallet, with the chosen UTXO and total spendable amount recorded.

## Explanation

A UTXO is an unspent transaction output controlled by a locking script. An outpoint is the unique coordinate of that output: the transaction id plus output index. A wallet balance is not a mutable account row; it is derived by scanning known UTXOs, filtering which are spendable and mature, and summing their values.
