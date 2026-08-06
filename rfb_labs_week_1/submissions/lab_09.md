# Lab 09 — Multi-UTXO coin selection

## Commands used

```bash
cargo test --test lab_09
bitcoin-cli -regtest createwallet "alice"
bitcoin-cli -regtest -rpcwallet=alice getnewaddress "alice-funding"
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "<ALICE_ADDRESS>" 0.4
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "<ALICE_ADDRESS>" 0.4
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "<ALICE_ADDRESS>" 0.4
bitcoin-cli -regtest generatetoaddress 1 "<MINER_ADDRESS>"
bitcoin-cli -regtest -rpcwallet=alice listunspent
bitcoin-cli -regtest -rpcwallet=receiver getnewaddress "alice-payment"
bitcoin-cli -regtest -rpcwallet=alice sendtoaddress "<NEW_RECEIVER_ADDRESS>" 1
bitcoin-cli -regtest getrawtransaction "<ALICE_SPEND_TXID>" 2
```

## Terminal output

```text
Alice address: [PASTE ACTUAL ADDRESS]
Three funding TXIDs/outpoints: [PASTE ACTUAL VALUES]
Confirmed Alice UTXOs: [PASTE RELEVANT LISTUNSPENT OUTPUT]
Combined spend TXID: [PASTE ACTUAL TXID]
Input count and consumed outpoints: [PASTE ACTUAL VALUES]
Receiver payment output: [PASTE ACTUAL 1 BTC OUTPUT]
Alice change output: [PASTE ACTUAL OUTPUT]
Fee: [PASTE CALCULATED VALUE]
Rust tests: [PASTE PASSING TEST SUMMARY]
```

## Evidence references

- [Alice's three confirmed 0.4 BTC UTXOs](evidence/lab_09_a.png)
- [Coin-selection transaction and its three inputs](evidence/lab_09_b.png)
- [Transaction outputs showing the 1 BTC payment and change](evidence/lab_09_c.png)

## Explanation

Alice cannot fund a 1 BTC payment with any single 0.4 BTC UTXO, so the wallet combines multiple UTXOs as transaction inputs. Each selected input is consumed completely. The receiver receives 1 BTC, surplus value returns to Alice as change, and the remaining difference is the miner fee. Combining inputs can reveal that the UTXOs are controlled by the same wallet, creating a privacy trade-off.
