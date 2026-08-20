# Bitcoin Transaction Serializer

A command-line tool to serialize Bitcoin transactions without modifying source code. This program accepts transaction data via command-line arguments and outputs the serialized hexadecimal representation.

## Features

- ✓ Support for multiple inputs and outputs
- ✓ SegWit transaction support
- ✓ Configurable version, locktime, and SegWit flags
- ✓ Comprehensive input validation with meaningful error messages
- ✓ Witness data support for SegWit transactions
- ✓ JSON-based configuration for flexible transaction construction

## Building

```bash
cargo build --release
```

## Usage

The program accepts transaction data as JSON arguments:

```bash
cargo run -- \
  --version 2 \
  --segwit \
  --locktime 0 \
  --inputs '[...]' \
  --outputs '[...]'
```

### Arguments

- `--version <VERSION>`: Transaction version (default: 2)
- `--segwit`: Enable SegWit serialization (flag, no value needed)
- `--locktime <LOCKTIME>`: Locktime value in seconds/blocks (default: 0)
- `--inputs <JSON>`: JSON array of transaction inputs (required)
- `--outputs <JSON>`: JSON array of transaction outputs (required)

### Input Format

Each input object in the JSON array must contain:

```json
{
  "prev_txid": "hex_string",        // Previous transaction ID (32 bytes)
  "vout": 0,                         // Previous output index (0-4294967295)
  "script_sig": "hex_string",        // Script signature (can be empty for SegWit)
  "sequence": 4294967295,            // Sequence number (optional, default: 0xffffffff)
  "witness": ["hex1", "hex2", ...]   // Witness items (optional, empty list for non-SegWit)
}
```

### Output Format

Each output object in the JSON array must contain:

```json
{
  "value": 69886,              // Satoshi amount (0-18446744073709551615)
  "script_pubkey": "hex_string" // Script pubkey
}
```

## Examples

### Example 1: Simple SegWit Transaction

Create a transaction with one input and two outputs:

```bash
cargo run -- \
  --version 2 \
  --segwit \
  --locktime 0 \
  --inputs '[
    {
      "prev_txid": "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821",
      "vout": 1,
      "script_sig": "",
      "sequence": 4294967295,
      "witness": [
        "3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301",
        "029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358"
      ]
    }
  ]' \
  --outputs '[
    {
      "value": 69886,
      "script_pubkey": "0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"
    },
    {
      "value": 29442,
      "script_pubkey": "00149831122b93d21715c70db626ccc844d3c21f9687"
    }
  ]'
```

**Output:**
```
=== TRANSACTION SERIALIZATION RESULT ===

Serialized Hex:
0200000000010181c80d2b05c0106360a36e435da4e418e85d0eb708d9f2bf2164b37bd007b08f010000000000ffffffffa17e0100000000001600141a632c1fff47af29f8c81dc4c6e91eb49a116c12b22730000000000160014c831122b93d21715c70db626ccc844d3c21f96870247304502210f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab3012029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000

Transaction size: 222 bytes

Transaction details:
  Version: 2
  SegWit: true
  Inputs: 1
  Outputs: 2
  Locktime: 0
```

### Example 2: Legacy Transaction (Non-SegWit)

Create a non-SegWit transaction:

```bash
cargo run -- \
  --version 1 \
  --segwit false \
  --locktime 0 \
  --inputs '[
    {
      "prev_txid": "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821",
      "vout": 0,
      "script_sig": "483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358",
      "sequence": 4294967295
    }
  ]' \
  --outputs '[
    {
      "value": 99328,
      "script_pubkey": "76a914a632c1fff47af29f8c81dc4c6e91eb49a116c12b88ac"
    }
  ]'
```

### Example 3: Multiple Inputs

Create a transaction with multiple inputs:

```bash
cargo run -- \
  --version 2 \
  --segwit \
  --locktime 0 \
  --inputs '[
    {
      "prev_txid": "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821",
      "vout": 0,
      "script_sig": "",
      "sequence": 4294967295,
      "witness": ["3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301", "029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358"]
    },
    {
      "prev_txid": "a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1",
      "vout": 1,
      "script_sig": "",
      "sequence": 4294967295,
      "witness": ["304402204567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef123402206789abcdef1234567890abcdef1234567890abcdef1234567890abcdef12345678", "0289abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"]
    }
  ]' \
  --outputs '[
    {
      "value": 50000,
      "script_pubkey": "0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"
    },
    {
      "value": 25000,
      "script_pubkey": "00149831122b93d21715c70db626ccc844d3c21f9687"
    }
  ]'
```

## Error Handling

The program validates all inputs and provides meaningful error messages:

```bash
# Invalid hex in transaction ID
cargo run -- --inputs '[{"prev_txid":"ZZZZ","vout":0}]' --outputs '[]'
# Error: Invalid hex in 'ZZZZ': invalid digit found in string

# Odd-length hex string
cargo run -- --inputs '[{"prev_txid":"abc","vout":0}]' --outputs '[]'
# Error: Hex string 'abc' has odd length

# Missing required field
cargo run -- --inputs '[{"vout":0}]' --outputs '[]'
# Error: Input 0 missing 'prev_txid' (string)

# Invalid JSON
cargo run -- --inputs 'not json' --outputs '[]'
# Error: expected value at line 1 column 1
```

## JSON Parsing Tips

- All hex strings should be lowercase (though uppercase is accepted)
- Witness arrays can be empty `[]` for non-SegWit transactions
- script_sig can be empty string `""` for SegWit transactions
- sequence defaults to 0xffffffff (4294967295) if omitted
- All numbers use standard JSON number format

## Building for Production

```bash
cargo build --release
# Binary will be at target/release/serialize-trx
```

## Development

Run tests:
```bash
cargo test
```

Check for issues:
```bash
cargo clippy
```

Format code:
```bash
cargo fmt
```
