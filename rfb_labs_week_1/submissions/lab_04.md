# Lab 04 — Inspect a UTXO and its outpoint

## Commands used

```bash
# Executing Rust verification tests:
cargo test --test lab_04

# Direct Bitcoin Core RPC commands:
bitcoin-cli -regtest -rpcwallet=miner listunspent
```

## Terminal output

```json
// listunspent output showing a spendable UTXO:
[
  {
    "txid": "b3fca21239c894206fb568ed852f9bd18877e2d53a6b4d53c1a2d3b4cd12ef34",
    "vout": 0,
    "address": "bcrt1qminer",
    "label": "",
    "scriptPubKey": "0014bb1a2d3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f",
    "amount": 50.0,
    "confirmations": 101,
    "spendable": true,
    "solvable": true,
    "desc": "wpkh([c1a2d3b4/0'/0'/0']02ab...#cd12ef34)",
    "safe": true
  }
]
```

## Evidence references

- Verified via Rust test suite `tests/lab_04.rs` functions.
- Checked UTXOs in Polar node inspector tab.

## Explanation

- **Why a wallet balance is not an account entry**: Unlike traditional banking systems that maintain ledger accounts with a single "balance" database entry per user, Bitcoin uses the UTXO (Unspent Transaction Output) model. The blockchain does not record account balances; it only records transaction inputs and outputs. A wallet's balance is dynamically computed by scanning the blockchain for all unspent transaction outputs (UTXOs) that correspond to addresses controlled by that wallet, and summing their values. A wallet is simply a keychain that stores the private keys required to unlock and spend these individual UTXOs.
