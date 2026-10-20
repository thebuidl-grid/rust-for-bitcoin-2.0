# Bitcoin Transaction Serializer (`serializeTrx`)

A command-line tool written in Rust to construct, validate, and serialize Bitcoin transactions (supporting both Legacy and SegWit formats with multiple inputs, outputs, and witness items) without modifying source code.

---

## Requirements & Building

Ensure you have Rust and Cargo installed.

```bash
cd rfb_labs_week_3/serializeTrx
cargo build
```

---

## Features

- **Dynamic CLI Input**: Pass transaction fields via command-line arguments (`--input`, `--output`, `--witness`, `--version`, `--locktime`, `--segwit`).
- **JSON Input Support**: Pass transaction definitions via a JSON file (`--file tx.json`) or JSON string (`--json '{...}'`).
- **Validation**: Validates hexadecimal strings, TXID lengths (32 bytes / 64 hex chars), integer parsing, and input/output bounds.
- **Serialization Output**: Prints the byte vector, serialized hexadecimal transaction string, and exact byte size.

---

## Usage Examples

### Example 1: SegWit Transaction via Command-Line Flags

```bash
cargo run -- \
  --version 2 \
  --locktime 0 \
  --input "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1:4294967295" \
  --witness "0:3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301,029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358" \
  --output "69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b" \
  --output "29442:00149831122b93d21715c70db626ccc844d3c21f9687"
```

**Output:**
```text
Serialized transaction:
[2, 0, 0, 0, 0, 1, 1, 143, 176, 208, 123, ...]
Serialized Hex transaction:
020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000

Transaction size: 223 bytes
```

---

### Example 2: Legacy Transaction via Command-Line Flags

```bash
cargo run -- \
  --version 1 \
  --locktime 0 \
  --input "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:0:4294967295:4730440220" \
  --output "50000:76a914111111111111111111111111111111111111111188ac"
```

---

### Example 3: Transaction via JSON String

```bash
cargo run -- --json '{
  "version": 2,
  "inputs": [
    {
      "prev_txid": "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821",
      "vout": 1,
      "sequence": 4294967295,
      "witness": [
        "3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301",
        "029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358"
      ]
    }
  ],
  "outputs": [
    {
      "value": 69886,
      "script_pubkey": "0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"
    },
    {
      "value": 29442,
      "script_pubkey": "00149831122b93d21715c70db626ccc844d3c21f9687"
    }
  ],
  "locktime": 0
}'
```

---

## Testing

Run the automated test suite:

```bash
cargo test
```
