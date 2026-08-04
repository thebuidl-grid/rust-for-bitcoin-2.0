# Lab 06 — Transaction decoding

## Commands used

TODO: Record the verbose transaction-decoding commands.
RECEIVER_ADDR=$(bitcoin-cli -regtest -rpcwallet=receiver getnewaddress)
echo $RECEIVER_ADDR

MINER_ADDR=$(bitcoin-cli -regtest -rpcwallet=miner getnewaddress)
echo $MINER_ADDR

TXID=$(bitcoin-cli -regtest -rpcwallet=miner sendtoaddress $RECEIVER_ADDR 1)
echo $TXID

bitcoin-cli -regtest -rpcwallet=miner generatetoaddress 1 $MINER_ADDR

bitcoin-cli -regtest getrawtransaction $TXID 2

## Terminal output

TODO: Include vin, vout, addresses, values, vsize, and calculated fee.
/Documents/rustforbitcoin/rust-for-bitcoin-2.0/rfb_labs_week_1$ cargo test --test lab_06
   Compiling rfb-labs-week-1 v0.1.0 (/home/jemiah/Documents/rustforbitcoin/rust-for-bitcoin-2.0/rfb_labs_week_1)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running tests/lab_06.rs (target/debug/deps/lab_06-f8e9a0592bd6c30e)

running 4 tests
test calculates_fee_from_input_and_output_values ... ok
test distinguishes_receiver_output_from_change ... ok
test returns_consumed_outpoints ... ok
test decodes_inputs_outputs_and_virtual_size ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


## Evidence references

TODO: Link screenshots or describe the attached evidence.
![alt text](image-5.png)

## Explanation

TODO: Prove value conservation and explain why the fee has no dedicated output.
Here's just that section, plain and ready to use:

---

**Value conservation:** A Bitcoin transaction can never create value out of nothing — the total value going out (all outputs) can never be more than the total value going in (all inputs). This lab proves it by calculating `sum(inputs) - sum(outputs)`. If that number were ever negative, it would mean the transaction created money that didn't exist, which a valid transaction can never do.

**Why the fee has no dedicated output:** The fee isn't sent to any address — it's just whatever value is left over after all the real outputs are paid. Whoever mines the block that includes this transaction automatically keeps that leftover amount as part of their reward. There's no separate "fee line" because the fee is just: inputs minus outputs.