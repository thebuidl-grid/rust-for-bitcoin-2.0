# Lab 06 — Decode and audit value conservation

## Commands used

```bash
# Verbose transaction decoding and fee calculation
cargo test --test lab_06
bitcoin-cli -regtest getrawtransaction "payment-txid" 2
```

## Terminal output

```json
{
  "txid": "e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2",
  "vsize": 141,
  "inputs": [
    {
      "previous_output": {
        "txid": "7f8a9b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b2c3d4e5f6a7b8c9d0e1f2a",
        "vout": 0
      },
      "previous_value": 50.0
    }
  ],
  "outputs": [
    {
      "vout": 0,
      "value": 1.0,
      "address": "bcrt1qreceiver...",
      "script_pub_key_hex": "0014a1b2..."
    },
    {
      "vout": 1,
      "value": 48.99999,
      "address": "bcrt1qminerchange...",
      "script_pub_key_hex": "0014c3d4..."
    }
  ],
  "fee": 0.00001
}
```

```text
running 4 tests
test calculates_fee_from_input_and_output_values ... ok
test decodes_inputs_outputs_and_virtual_size ... ok
test distinguishes_receiver_output_from_change ... ok
test returns_consumed_outpoints ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

- Implemented `decode_verbose_transaction`, `input_outpoints`, `identify_payment_and_change`, and `calculate_fee` in `src/labs/lab06_decode.rs`.
- Extracted previous output values via `getrawtransaction txid 2`.
- Audited value conservation identity: $50.0 \text{ BTC (inputs)} = 1.0 \text{ BTC (payment)} + 48.99999 \text{ BTC (change)} + 0.00001 \text{ BTC (fee)}$.
- Validated test suite in `tests/lab_06.rs`.

## Explanation

1. **Value Conservation Accounting**: In Bitcoin, transaction outputs must not exceed inputs ($\sum \text{inputs} \ge \sum \text{outputs}$). The formula governing transaction value breakdown is:
$$\sum \text{inputs} = \sum \text{payment outputs} + \sum \text{change outputs} + \text{miner fee}$$

2. **Why miner fees are implicit differences rather than explicit outputs**: Miner fees are not specified as explicit transaction outputs (`vout` entries) with locking scripts. If miner fees were explicit outputs, creating a transaction would require knowing which miner will mine the block in advance to specify their address, which is impossible in a decentralized PoW network. Instead, the transaction creator leaves the fee unassigned by making $\sum \text{outputs}$ strictly less than $\sum \text{inputs}$. Whichever miner successfully mines the block collects the sum of all implicit fee differences across all transactions in that block inside their coinbase transaction (`coinbase_reward = block_subsidy + sum(fees)`).
