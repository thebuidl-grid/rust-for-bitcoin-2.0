# Bitcoin Transaction Serializer

A command-line tool written in Rust that serializes Bitcoin transactions without requiring hardcoded transaction data. This tool allows you to construct and serialize Bitcoin transactions with custom inputs, outputs, and witness data directly from the command line.

## Features

- **Flexible Command-Line Interface**: Provide transaction data through CLI arguments
- **SegWit Support**: Serialize both legacy and SegWit transactions
- **Multiple Inputs/Outputs**: Support for transactions with multiple inputs and outputs
- **Witness Data Support**: Include witness data for SegWit transactions
- **Input Validation**: Comprehensive validation of transaction data (hex format, field sizes, etc.)
- **Clear Output**: Display serialized transaction in hexadecimal with size information

## Requirements

- Rust 1.70 or later
- Cargo

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run -- [OPTIONS]
```

Or use the compiled binary:

```bash
./target/release/serializeTrx [OPTIONS]
```

## Command-Line Options

### Core Options

- `--version <VERSION>`: Transaction version (default: 2)
- `--segwit`: Enable SegWit serialization (flag, default: disabled)
- `--locktime <LOCKTIME>`: Transaction locktime (default: 0)
- `--input <JSON>`: Transaction input in JSON format (can be used multiple times)
- `--output <JSON>`: Transaction output in JSON format (can be used multiple times)

### Input JSON Format

Each input must be provided as a JSON string with the following structure:

```json
{
  "prev_txid": "32-byte hex string (64 hex characters)",
  "vout": 0,
  "script_sig": "hex string or empty string",
  "sequence": 4294967295,
  "witness": ["signature hex", "pubkey hex"]
}
```

**Field Descriptions:**
- `prev_txid`: The previous transaction ID (64 hex characters representing 32 bytes)
- `vout`: The output index from the previous transaction (0-based)
- `script_sig`: The script signature (empty for SegWit inputs)
- `sequence`: The sequence number (default: 0xffffffff = 4294967295)
- `witness`: Array of witness data (for SegWit transactions)

### Output JSON Format

Each output must be provided as a JSON string with the following structure:

```json
{
  "value": 50000,
  "script_pubkey": "hex string"
}
```

**Field Descriptions:**
- `value`: The amount in satoshis
- `script_pubkey`: The script pubkey (hex string)

## Examples

### Example 1: Simple Legacy Transaction

A basic transaction with one input and one output (non-SegWit):

```bash
cargo run -- \
  --version 2 \
  --locktime 0 \
  --input '{"prev_txid":"8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821","vout":1,"script_sig":"","sequence":4294967295,"witness":[]}' \
  --output '{"value":69886,"script_pubkey":"0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"}'
```

### Example 2: SegWit Transaction with Witness Data

A SegWit transaction with witness data (like the original hardcoded example):

```bash
cargo run -- \
  --version 2 \
  --segwit \
  --locktime 0 \
  --input '{"prev_txid":"8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821","vout":1,"script_sig":"","sequence":4294967295,"witness":["3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301","029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358"]}' \
  --output '{"value":69886,"script_pubkey":"0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"}' \
  --output '{"value":29442,"script_pubkey":"00149831122b93d21715c70db626ccc844d3c21f9687"}'
```

### Example 3: Multi-Input, Multi-Output Transaction

A transaction with multiple inputs and outputs:

```bash
cargo run -- \
  --version 2 \
  --segwit \
  --locktime 0 \
  --input '{"prev_txid":"8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821","vout":0,"script_sig":"","sequence":4294967295,"witness":[]}' \
  --input '{"prev_txid":"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef","vout":1,"script_sig":"","sequence":4294967295,"witness":[]}' \
  --output '{"value":50000,"script_pubkey":"0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"}' \
  --output '{"value":40000,"script_pubkey":"00149831122b93d21715c70db626ccc844d3c21f9687"}' \
  --output '{"value":10000,"script_pubkey":"0014aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}'
```

### Example 4: Custom Version and Locktime

A transaction with a custom version and locktime:

```bash
cargo run -- \
  --version 1 \
  --locktime 500000 \
  --input '{"prev_txid":"8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821","vout":0,"script_sig":"","sequence":0,"witness":[]}' \
  --output '{"value":100000,"script_pubkey":"76a914a632c1fff47af29f8c81dc4c6e91eb49a116c12b88ac"}'
```

## Output Format

The program outputs:

1. **Transaction Details**: Version, SegWit status, input/output counts, and locktime
2. **Serialized Transaction**: The complete transaction in hexadecimal format
3. **Transaction Size**: The size in bytes

Example output:

```
=== Bitcoin Transaction Serialization ===

Transaction Details:
  Version: 2
  SegWit: true
  Input Count: 1
  Output Count: 2
  Locktime: 0

Serialized Transaction (Hex):
02000000000101...

Transaction Size: 222 bytes
```

## Validation

The program validates:

- **Hexadecimal Format**: All hex strings must contain only valid hex characters
- **Previous Transaction ID**: Must be exactly 64 hex characters (32 bytes)
- **Value Ranges**: All numeric fields must fit within their designated types
- **Transaction Structure**: Must have at least one input and one output
- **Witness Data**: Only included when SegWit flag is enabled

### Error Examples

Missing required fields:
```bash
cargo run -- --input '{"vout":1}'
# Error: Missing or invalid prev_txid
```

Invalid hex string:
```bash
cargo run -- \
  --input '{"prev_txid":"zzzz","vout":0,"script_sig":"","sequence":4294967295,"witness":[]}' \
  --output '{"value":1000,"script_pubkey":"0014aaa"}'
# Error: invalid digit found in string
```

Wrong prev_txid length:
```bash
cargo run -- \
  --input '{"prev_txid":"1234","vout":0,"script_sig":"","sequence":4294967295,"witness":[]}' \
  --output '{"value":1000,"script_pubkey":"0014aaa"}'
# Error: prev_txid must be exactly 32 bytes (64 hex characters)
```

## JSON Input Tips

When passing JSON strings on the command line, be careful with quotes:

**Option 1: Single quotes (recommended for most shells)**
```bash
cargo run -- --input '{"prev_txid":"...","vout":0,"script_sig":"","sequence":4294967295,"witness":[]}'
```

**Option 2: Double quotes with escaping (for shells that require it)**
```bash
cargo run -- --input "{\"prev_txid\":\"...\",\"vout\":0,\"script_sig\":\"\",\"sequence\":4294967295,\"witness\":[]}"
```

**Option 3: Using a JSON file**

Create a file `input.json`:
```json
{
  "prev_txid": "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821",
  "vout": 1,
  "script_sig": "",
  "sequence": 4294967295,
  "witness": []
}
```

Then pass it with command substitution (bash/zsh):
```bash
cargo run -- --input "$(cat input.json)" --output '{"value":69886,"script_pubkey":"0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"}'
```

## Architecture

### Core Structures

- **TxInput**: Represents a transaction input with prev_txid, vout, script_sig, sequence, and witness data
- **TxOutput**: Represents a transaction output with value and script_pubkey
- **Transaction**: Complete transaction structure with version, inputs, outputs, locktime, and segwit flag

### Key Functions

- `hex_to_bytes()`: Converts hex strings to byte vectors with validation
- `bytes_to_hex()`: Converts byte vectors to hex strings
- `parse_input_json()`: Parses and validates input JSON
- `parse_output_json()`: Parses and validates output JSON
- `serialize_transaction()`: Serializes transaction according to Bitcoin protocol
- `encode_varint()`: Encodes variable-length integers (CompactSize)

## Transaction Structure

The serialized transaction follows the Bitcoin protocol:

**Non-SegWit:**
```
Version (4 bytes)
  → Input Count (VarInt)
    → Inputs (variable)
  → Output Count (VarInt)
    → Outputs (variable)
  → Locktime (4 bytes)
```

**SegWit:**
```
Version (4 bytes)
Marker (1 byte) = 0x00
Flag (1 byte) = 0x01
  → Input Count (VarInt)
    → Inputs (variable)
  → Output Count (VarInt)
    → Outputs (variable)
  → Witness Data (variable)
  → Locktime (4 bytes)
```

## Testing

Run the included examples:

```bash
# Basic SegWit transaction
./run_example_1.sh

# Multi-input transaction
./run_example_2.sh

# Custom transaction
./run_example_3.sh
```

See `examples/` directory for more sample scripts.

## Troubleshooting

### "Transaction must have at least one input"
Make sure to provide at least one `--input` argument.

### "Invalid hex string"
Verify all hex strings contain only characters 0-9 and a-f.

### "prev_txid must be exactly 32 bytes"
Previous transaction IDs must be exactly 64 hexadecimal characters.

### JSON parsing errors
Ensure JSON strings are properly formatted and all required fields are present.

## License

This project is part of the Rust for Bitcoin course.

## Refactoring Summary

This tool refactors the original hardcoded transaction serializer to:

1. **Accept flexible CLI arguments** - Users can construct any valid Bitcoin transaction
2. **Support multiple inputs/outputs** - Not limited to specific transaction structures
3. **Include comprehensive validation** - Prevents invalid transactions
4. **Maintain compatibility** - Produces identical serialization to the original program
5. **Improve usability** - Clear error messages and helpful documentation

The refactored program preserves all the core serialization logic while making the tool flexible enough for real-world use cases.
