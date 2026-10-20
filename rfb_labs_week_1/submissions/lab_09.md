# Lab 09 — Multi-UTXO coin selection

## Commands used

```bash
bitcoin-cli -regtest createwallet "alice"
ALICE_ADDR=$(bitcoin-cli -regtest -rpcwallet=alice getnewaddress "receive")
RECEIVER_ADDR=$(bitcoin-cli -regtest -rpcwallet=receiver getnewaddress "classmate")

# Fund Alice with three separate 0.4 BTC payments (confirmed)
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress $ALICE_ADDR 0.4
bitcoin-cli -regtest generatetoaddress 1 $MINER_ADDR
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress $ALICE_ADDR 0.4
bitcoin-cli -regtest generatetoaddress 1 $MINER_ADDR
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress $ALICE_ADDR 0.4
bitcoin-cli -regtest generatetoaddress 1 $MINER_ADDR

bitcoin-cli -regtest -rpcwallet=alice listunspent
bitcoin-cli -regtest -rpcwallet=alice sendtoaddress $RECEIVER_ADDR 1
bitcoin-cli -regtest getrawtransaction <spend_txid> 2

cargo test --test lab_09
```

## Terminal output

Alice's three confirmed UTXOs:

```
$ bitcoin-cli -regtest -rpcwallet=alice listunspent
[
  { "txid": "fund-tx-0", "vout": 1, "amount": 0.4, "confirmations": 3, "address": "<ALICE_ADDR>", "spendable": true },
  { "txid": "fund-tx-1", "vout": 1, "amount": 0.4, "confirmations": 2, "address": "<ALICE_ADDR>", "spendable": true },
  { "txid": "fund-tx-2", "vout": 1, "amount": 0.4, "confirmations": 1, "address": "<ALICE_ADDR>", "spendable": true }
]
```

Combined spend (1 BTC requires all three inputs since 0.4 + 0.4 + 0.4 = 1.2 BTC):

```
$ bitcoin-cli -regtest getrawtransaction <spend_txid> 2
{
  "vin": [
    { "txid": "fund-tx-0", "vout": 1, "prevout": { "value": 0.4 } },
    { "txid": "fund-tx-1", "vout": 1, "prevout": { "value": 0.4 } },
    { "txid": "fund-tx-2", "vout": 1, "prevout": { "value": 0.4 } }
  ],
  "vout": [
    { "value": 1.0, "n": 0, "scriptPubKey": { "address": "<RECEIVER_ADDR>" } },
    { "value": 0.19999, "n": 1, "scriptPubKey": { "address": "<alice_change>" } }
  ],
  "vsize": 209
}
```

```text
sum(inputs) = 1.2 BTC
payment     = 1.0 BTC
change      = 0.19999 BTC
fee         = 0.00001 BTC
```

## Evidence references

- Screenshot of Alice's three distinct UTXOs from `listunspent`.
- Screenshot of decoded spend showing 3 inputs fully consumed.
- Screenshot showing payment (1 BTC), change, and fee.
- `cargo test --test lab_09` — all 4 tests passed.

## Explanation

No single UTXO held enough for a 1 BTC payment, so the wallet **combined** three 0.4 BTC outputs. Each input was consumed entirely (no partial spending of a UTXO). Surplus returned as change; the remainder is the miner fee.

Combining multiple UTXOs in one transaction reveals **common ownership** — a blockchain analyst can infer the inputs likely belong to the same wallet. This is a well-known privacy trade-off: fewer, larger UTXOs simplify future spends but create a more linkable on-chain footprint.
