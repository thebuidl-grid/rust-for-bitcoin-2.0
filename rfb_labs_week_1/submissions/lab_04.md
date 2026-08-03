# Lab 04 — UTXOs and outpoints

## Commands used

```bash
cargo test --test lab_04
```

RPC methods called:
- `listunspent` - List all unspent transaction outputs (UTXOs) in wallet context

## Terminal output

```
running 4 tests
test constructs_unique_outpoint ... ok
test decodes_listunspent_response ... ok
test selects_most_confirmed_spendable_utxo ... ok
test sums_only_spendable_outputs ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All tests pass successfully, demonstrating:
- Parsing `listunspent` JSON response with all UTXO fields
- Filtering UTXOs by spendability and address
- Selecting the most-confirmed UTXO for deterministic coin selection
- Summing only spendable UTXOs for balance calculation

## Explanation

Lab 04 introduces the UTXO (Unspent Transaction Output) model, fundamental to Bitcoin:

1. **UTXOs**: Every Bitcoin transaction consumes inputs (spent outputs) and creates outputs. Unspent outputs are "coins" in the wallet. Each UTXO is identified by:
   - **txid**: Transaction ID containing the output
   - **vout**: Output index (0-based) within that transaction
   - **amount**: BTC value
   - **confirmations**: Number of blocks confirming it
   - **spendable**: Whether wallet can spend it

2. **Outpoints**: A `txid:vout` pair uniquely identifies a UTXO in the entire blockchain. This is how transactions reference which outputs they're spending.

3. **Wallet Balance**: A wallet's balance is the sum of all its spendable UTXOs. This is why Bitcoin transactions must "use up" entire UTXOs - you can't spend part of one.

4. **Coin Selection**: When creating transactions, wallets must select which UTXOs to spend. This lab demonstrates prioritizing UTXOs with more confirmations, which is a common coin selection strategy.
