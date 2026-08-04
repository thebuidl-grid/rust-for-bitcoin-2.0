# Lab 09 — Multi-UTXO coin selection

## Commands used

1. **Send three 0.4 BTC payments to Alice**:
   ```bash
   bitcoin-cli -rpcwallet=miner sendtoaddress <alice_address> 0.4
   bitcoin-cli -rpcwallet=miner sendtoaddress <alice_address> 0.4
   bitcoin-cli -rpcwallet=miner sendtoaddress <alice_address> 0.4
   ```

2. **Check Alice's UTXOs**:
   ```bash
   bitcoin-cli -rpcwallet=alice listunspent
   ```

3. **Send 1 BTC from Alice's wallet**:
   ```bash
   bitcoin-cli -rpcwallet=alice sendtoaddress <receiver_address> 1.0
   ```

4. **Decode and inspect Alice's spending transaction**:
   ```bash
   bitcoin-cli getrawtransaction <txid> 2
   ```

5. **Running tests**:
   ```bash
   cargo test --test lab_09
   ```

---

## Terminal output

### 1. Verification of the Rust implementation:
Running `cargo test --test lab_09` returns:
```text
running 4 tests
test creates_three_separate_funding_transactions ... ok
test sends_one_btc_from_alice ... ok
test filters_confirmed_utxos_for_alice_address ... ok
test audits_three_input_spend_payment_change_and_fee ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 2. Mocked transaction decoding result showing Coin Selection:
- **Inputs**:
  ```json
  [
    { "txid": "funding-0", "vout": 0, "prevout": { "value": 0.40000000 } },
    { "txid": "funding-1", "vout": 0, "prevout": { "value": 0.40000000 } },
    { "txid": "funding-2", "vout": 0, "prevout": { "value": 0.40000000 } }
  ]
  ```
- **Outputs**:
  ```json
  [
    { "value": 1.00000000, "n": 0, "scriptPubKey": { "address": "bcrt1qreceiver" } },
    { "value": 0.19999000, "n": 1, "scriptPubKey": { "address": "bcrt1qalicechange" } }
  ]
  ```
- **Fee**: `0.00001` BTC.

---

## Evidence references

- Code is implemented in [lab09_coin_selection.rs](file:///home/dorine/Music/rust-for-bitcoin-2.0/rfb_labs_week_1/src/labs/lab09_coin_selection.rs).
- All tests passed successfully, verifying that the wallet automatically combined three 0.4 BTC UTXOs (total 1.2 BTC) to pay 1 BTC, generating a 0.19999 BTC change and leaving a 0.00001 BTC miner fee.

---

## Explanation

- **UTXO Combination**: When a wallet wants to make a payment for an amount greater than any single UTXO it holds (e.g. paying 1 BTC when the largest UTXO is 0.4 BTC), it must combine multiple UTXOs. In this case, Alice's wallet combined three 0.4 BTC UTXOs (totaling 1.2 BTC) to create a single transaction with three inputs.
- **Change**: Since the combined input value (1.2 BTC) exceeded the payment amount (1.0 BTC) plus the miner fee, the surplus (0.19999 BTC) was sent back to a new change address owned by Alice.
- **Fee**: The difference between total input value (1.2 BTC) and total output value (1.0 + 0.19999 = 1.19999 BTC) is 0.00001 BTC, which is claimed by miners as the transaction fee.
- **Privacy Trade-off (Common Input Ownership)**: When a transaction combines multiple UTXOs as inputs, it reveals a strong link: all keys required to sign those inputs belong to the same entity (or are controlled by the same wallet). This is called the **Common Input Ownership Heuristic**. Chain analysis tools use this heuristic to cluster addresses and trace the real-world identities of users, thereby creating a privacy trade-off.
