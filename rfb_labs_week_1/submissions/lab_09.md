# Lab 09 — Multi-UTXO coin selection

## Commands used

```bash
cargo test --test lab_09
bitcoin-cli -regtest createwallet alice
bitcoin-cli -regtest -rpcwallet=alice getnewaddress alice
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress <alice-address> 0.4
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress <alice-address> 0.4
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress <alice-address> 0.4
bitcoin-cli -regtest generatetoaddress 1 <miner-address>
bitcoin-cli -regtest -rpcwallet=alice listunspent
bitcoin-cli -regtest -rpcwallet=alice sendtoaddress <receiver-address> 1
bitcoin-cli -regtest getrawtransaction <combined-spend-txid> 2
```

## Terminal output

Alice owned three distinct confirmed 0.4 BTC UTXOs. Sending 1 BTC required more than one input, consumed selected inputs completely, created a 1 BTC receiver output, returned surplus as change, and left the input/output difference as the miner fee.

## Evidence references

Evidence is the Lab 09 test run, Alice's `listunspent` output showing the three funding outpoints, and the verbose decoded combined spend showing multiple inputs, receiver output, change output, and fee.

## Explanation

UTXOs are indivisible at spend time: each selected input is consumed in full, and any surplus must be recreated as change. Combining several UTXOs in one transaction can reveal that the same spender likely controls all selected inputs, so coin selection has a privacy trade-off as well as a fee and convenience trade-off.
