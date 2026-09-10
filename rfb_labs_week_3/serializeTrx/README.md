# Bitcoin Transaction Serializer

This Rust program serializes Bitcoin transaction data provided via command-line arguments into a hexadecimal string.

## How to Run

1. Ensure you have Rust and Cargo installed.
2. Navigate to the project directory.
3. Run the program using `cargo run --` followed by the required arguments.

## Command Line Arguments

- `-v, --tx-version`: Transaction version (integer).
- `-s, --segwit`: Enable SegWit (flag).
- `-i, --input`: Input hex string. Use multiple times for multiple inputs.
- `-o, --output`: Output in `value:script_hex` format. Use multiple times for multiple outputs.
- `-w, --witness`: Witness hex string. Use multiple times for multiple witness items.
- `-l, --locktime`: Locktime (integer).

## Examples

### Example 1: Simple Legacy Transaction
A transaction with one input, one output, and no SegWit.

```bash
cargo run -- \
  --tx-version 2 \
  --input "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef:0:483045022100abcd1234:ffffffff" \
  --output "40000:76a91488ac" \
  --locktime 0
```

### Example 2: Legacy Transaction with multiple inputs and outputs
A transaction with one input, one output, and no SegWit.

```bash
cargo run -- \
  --tx-version 2 \
  --input "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef:0:483045022100abcd1234:ffffffff" \
  --input "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890:1:483045022100efda5678:ffffffff" \
  --output "40000:76a91488ac" \
  --output "9000:76a91499bd" \
  --locktime 0

```

### Example 3: Simple Segwit Transaction

```bash
cargo run -- \
  --segwit \
  --input "0000000000000000000000000000000000000000000000000000000000000000:0" \
  --output "50000:76a91488ac" \
  --witness "3045022100abcd1234" \
  --witness "02abcd1234" \
  --witness-counts 2 \
  --locktime 0
``` 

### Example 4: Multi-Input/Output SegWit Transaction


```bash
cargo run -- \
  --segwit \
  --tx-version 2 \
  --input "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef:0:483045022100abcd1234:ffffffff" \
  --input "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890:1::feffffff" \
  --output "40000:76a91412ab34cd56ef78901234567890abcdef12345678ac" \
  --output "9000:001412ab34cd56ef78901234567890abcdef12345678" \
  --witness "3045022100abcd1234567890abcdef1234567890abcd1234567890abcdef1234567890abcd" \
  --witness "02abcd1234567890abcdef1234567890abcd1234567890abcdef1234567890abcd" \
  --witness "3045022100efda5678901234567890abcdef1234567890efda5678901234567890abcdefda" \
  --witness "03efda5678901234567890abcdef1234567890efda5678901234567890abcdefda" \
  --witness-counts 2,2 \
  --locktime 100000
```
