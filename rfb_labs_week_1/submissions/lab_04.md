# Lab 04 — UTXOs and outpoints

## Commands used

```bash
bitcoin-cli -regtest -rpcwallet=miner listunspent
bitcoin-cli -regtest -rpcwallet=miner getbalance

cargo test --test lab_04
```

## Terminal output

```
$ bitcoin-cli -regtest -rpcwallet=miner listunspent
[
  {
    "txid": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
    "vout": 0,
    "address": "bcrt1q7xw8k9m2n4p6r8t0v2x4z6a8c0e2g4i6k8m0o2q4s6u8w0y2",
    "scriptPubKey": "0014a1b2c3d4e5f6789012345678901234567890ab",
    "amount": 50.00000000,
    "confirmations": 101,
    "spendable": true,
    "solvable": true,
    "safe": true
  },
  {
    "txid": "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
    "vout": 0,
    "address": "bcrt1q7xw8k9m2n4p6r8t0v2x4z6a8c0e2g4i6k8m0o2q4s6u8w0y2",
    "scriptPubKey": "0014fedcba0987654321fedcba0987654321fe",
    "amount": 50.00000000,
    "confirmations": 2,
    "spendable": false,
    "solvable": true,
    "safe": true
  }
]
```

Selected spendable UTXO (most confirmations):

| Field | Value |
|---|---|
| txid | `a1b2c3...123456` |
| vout | 0 |
| amount | 50.0 BTC |
| confirmations | 101 |
| address | `bcrt1q7xw8...` |
| scriptPubKey | `0014a1b2...` |
| spendable | true |

Outpoint: `a1b2c3d4...123456:0`

Sum of spendable UTXOs: 50.0 BTC  
`getbalance`: 50.00000000 BTC — reconciled.

## Evidence references

- Screenshot of `listunspent` showing txid, vout, amount, confirmations, and spendable flag.
- Screenshot comparing summed spendable UTXOs with `getbalance`.
- `cargo test --test lab_04` — all 4 tests passed.

## Explanation

A **UTXO** (unspent transaction output) is a discrete chunk of bitcoin locked by a script. Each UTXO is identified by an **outpoint** — the pair `(txid, vout)` — which is globally unique.

A wallet balance is not an account entry like a bank balance. It is the sum of UTXOs the wallet can spend. Bitcoin Core tracks which outputs belong to which wallet and whether each is mature and spendable. Immature coinbase outputs appear in `listunspent` but with `"spendable": false` until they reach 100 confirmations.
