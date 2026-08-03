# Lab 09 — Multi-UTXO coin selection

## Commands used

```bash
cargo test --test lab_09
```

RPC methods called:
- `sendtoaddress` - Send funding transactions (three 0.4 BTC)
- `listunspent` - List Alice's UTXOs
- `sendtoaddress` - Create payment requiring multiple inputs
- `getrawtransaction <txid> 2` - Decode multi-input transaction

## Terminal output

```
running 4 tests
test audits_three_input_spend_payment_change_and_fee ... ok
test creates_three_separate_funding_transactions ... ok
test filters_confirmed_utxos_for_alice_address ... ok
test sends_one_btc_from_alice ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All tests pass successfully, demonstrating:
- Creating multiple UTXOs (three 0.4 BTC outputs)
- Filtering confirmed UTXOs for specific addresses
- Wallet automatically combining multiple UTXOs (1.2 BTC) to create 1.0 BTC payment
- Decoded transaction shows 3 inputs, 2 outputs (payment + change), and fee calculation

## Explanation

Lab 09 demonstrates coin selection - how wallets combine UTXOs to create transactions:

1. **Multi-UTXO Spending**: When a single UTXO is insufficient, wallets combine multiple outputs. Alice has three 0.4 BTC outputs but needs to send 1.0 BTC, so the wallet uses all three.

2. **Coin Selection Algorithm**: Wallets use strategies like:
   - **Largest First**: Select outputs in descending value order
   - **Oldest First**: Prioritize older outputs with more confirmations
   - **UTXO Consolidation**: Combine many small outputs into fewer large ones

3. **Change Output**: Spending 1.2 BTC input for 1.0 BTC payment requires a change output (0.2 BTC minus fees). This change typically goes to an address Alice controls, often a newly generated address.

4. **Fee Calculation**: Transaction fee = 1.2 - 1.0 - change_amount. The 0.00001 BTC fee incentivizes miners to include this transaction.

5. **Privacy Implications**: Observing which UTXOs appear in the same transaction reveals they're controlled by the same wallet. This is a privacy concern - external observers can link addresses together through transaction analysis.

6. **Efficiency**: Combining multiple inputs increases transaction size and fees. Wallets balance between having manageable UTXO counts and minimizing fees. This is why UTXO consolidation during low-fee periods is recommended.
