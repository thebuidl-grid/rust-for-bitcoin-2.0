# Lab 06 — Transaction decoding

## Commands used

```bash
cargo test --test lab_06
```

RPC methods called:
- `getrawtransaction <txid> 2` - Decode transaction with verbosity level 2 (includes input values)

## Terminal output

```
running 4 tests
test calculates_fee_from_input_and_output_values ... ok
test decodes_inputs_outputs_and_virtual_size ... ok
test distinguishes_receiver_output_from_change ... ok
test returns_consumed_outpoints ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All tests pass successfully, demonstrating:
- Parsing `getrawtransaction` verbose output with nested JSON structures
- Extracting input values, output values, and script pubkeys
- Calculating transaction fees as input sum minus output sum
- Identifying payment vs change outputs by address

## Explanation

Lab 06 demonstrates transaction value analysis - critical for understanding Bitcoin's value flow:

1. **Transaction Structure**:
   - **Inputs (vin)**: Previous outputs being spent, each references a prior transaction's output
   - **Outputs (vout)**: New outputs created, each specifying amount and recipient script
   - **Fee**: Implicit = sum(inputs) - sum(outputs). Miners claim this as payment.

2. **Value Conservation**: Bitcoin enforces strict value conservation - all input value must be accounted for in outputs or fees. This is verified by every node.

3. **Change Outputs**: When spending, you typically use inputs that exceed the amount you want to send. The excess becomes a change output back to your wallet. This is how transactions have multiple outputs.

4. **No Fee Output**: Unlike some payment systems, Bitcoin has no explicit fee output. The miner implicitly receives any difference between input and output values. This design has security implications - miners are incentivized to maximize fees.

5. **Virtual Size (vsize)**: Modern transactions use vsize (virtual size) for fee calculation, accounting for witness data in SegWit transactions. This affects how fees are calculated per byte.
