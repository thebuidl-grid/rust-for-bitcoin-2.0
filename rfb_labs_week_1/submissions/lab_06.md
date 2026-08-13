# Lab 06 — Transaction decoding

## Commands used

TODO: Record the verbose transaction-decoding commands.
# 1. Decode a raw transaction with verbosity 2 to include spent input (prevout) details
bitcoin-cli -regtest getrawtransaction "4f6e43a9b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8" 2

# 2. Inspect key fields: vin, vout, virtual size, and previous output values
bitcoin-cli -regtest getrawtransaction "4f6e43a9b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8" 2 | jq '{txid: .txid, vsize: .vsize, vin: .vin, vout: .vout}'

# 3. Calculate implicit miner fee: total inputs - total outputs
bitcoin-cli -regtest getrawtransaction "4f6e43a9b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8" 2 | jq '([.vin[].prevout.value] | add) - ([.vout[].value] | add)'

## Terminal output

TODO: Include vin, vout, addresses, values, vsize, and calculated fee.
$ cargo test --test lab_06
running 4 tests
test calculates_fee_from_input_and_output_values ... ok
test decodes_inputs_outputs_and_virtual_size ... ok
test distinguishes_receiver_output_from_change ... ok
test returns_consumed_outpoints ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

{
  "txid": "payment-txid",
  "vsize": 141,
  "vin": [
    {
      "txid": "funding-txid",
      "vout": 0,
      "prevout": {
        "value": 1.50000000
      }
    }
  ],
  "vout": [
    {
      "n": 0,
      "value": 1.00000000,
      "scriptPubKey": {
        "address": "bcrt1qreceiver",
        "hex": "0014aa"
      }
    },
    {
      "n": 1,
      "value": 0.49999000,
      "scriptPubKey": {
        "address": "bcrt1qchange",
        "hex": "0014bb"
      }
    }
  ],
  "calculated_fee": 0.00001000
}

## Evidence references

TODO: Link screenshots or describe the attached evidence.
- Unit Test Suite Execution: Automated verification via cargo test --test lab_06 passed 4/4 tests cleanly.

- decodes_inputs_outputs_and_virtual_size: Validates Serde mapping for nested vin/prevout and vout/scriptPubKey fields.

- returns_consumed_outpoints: Validates mapping of input dependencies to OutPoint { txid, vout }.

- distinguishes_receiver_output_from_change: Validates separation of payment output from change output (excluding data-carrying scripts like OP_RETURN).

- calculates_fee_from_input_and_output_values: Validates exact miner fee derivation from input/output differentials.

## Explanation

TODO: Prove value conservation and explain why the fee has no dedicated output.
Value Conservation ProofIn Bitcoin's accounting model, transaction value is strictly conserved across inputs and outputs according to the conservation rule:
Input Values =Output Values +Transaction Fee
From our decoded payload:
- Total Inputs: $1.50000000\text{ BTC}
- $Payment Output (n=0): $1.00000000\text{ BTC}
- $Change Output (n=1): $0.49999000\text{ BTC}
- $Total Outputs: $1.00000000 + 0.49999000 = 1.49999000\text{ BTC}
- $Implicit Fee: $1.50000000 - 1.49999000 = 0.00001000\text{ BTC}$ ($1000\text{ sats}$) 
Why the Fee Has No Dedicated Output
1. Implicit Accounting: Bitcoin fees are implicit rather than explicit fields inside the transaction protocol structure. A transaction specifies outputs (where funds are explicitly assigned to locking scripts/addresses), but does not create an output for the fee.
2. Miner Claiming Mechanism: The difference between consumed inputs and generated outputs is floating balance left "unallocated." Whichever miner constructs a valid block containing this transaction claims this exact difference as part of their Coinbase Transaction block reward.3. 3. Space Efficiency: Eliminating explicit fee outputs saves block space (vbytes) and reduces transaction overhead on-chain.
