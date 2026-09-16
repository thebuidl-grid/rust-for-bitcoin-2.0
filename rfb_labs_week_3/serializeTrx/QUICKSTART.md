# Quick Start Guide

## Installation

```bash
cd rfb_labs_week_3/serializeTrx
cargo build --release
```

The binary will be at `target/release/serialize-trx`.

## Basic Usage Pattern

All transactions require two JSON arrays: inputs and outputs.

### Minimal SegWit Transaction

```bash
./target/release/serialize-trx \
  --segwit \
  --inputs '[{
    "prev_txid": "<32-byte-hex-txid>",
    "vout": 0,
    "script_sig": "",
    "witness": ["<signature-hex>", "<pubkey-hex>"]
  }]' \
  --outputs '[{
    "value": 50000,
    "script_pubkey": "<script-hex>"
  }]'
```

### Minimal Non-SegWit Transaction

```bash
./target/release/serialize-trx \
  --inputs '[{
    "prev_txid": "<32-byte-hex-txid>",
    "vout": 0,
    "script_sig": "<signature-pubkey-hex>"
  }]' \
  --outputs '[{
    "value": 50000,
    "script_pubkey": "<script-hex>"
  }]'
```

## Common Values Reference

### Sequence Numbers
- `4294967295` (0xffffffff): Final/default - transaction can be included immediately
- `0`: Transaction locked until locktime
- `1-4294967294`: Relative lock time (depends on transaction version)

### Locktime
- `0`: No locktime restriction
- `1-500,000,000`: Block height (Bitcoin block numbers)
- `500,000,001+`: Unix timestamp (seconds since Jan 1, 1970)

### Script Types
- P2PKH: `76a914<pubkey-hash>88ac`
- P2WPKH: `0014<pubkey-hash>` (20 bytes, 40 hex chars)
- P2SH: `a914<script-hash>87`
- P2WSH: `0020<script-hash>` (32 bytes, 64 hex chars)

## Tips

1. **Hex Validation**: All hex strings are validated automatically
   - Must be even length (even number of hex digits)
   - Only 0-9, a-f, A-F characters allowed
   - Leading zeros are preserved

2. **Transaction IDs**: 
   - Must be 32 bytes (64 hex characters) after validation
   - Usually provided in reversed format in blockchain explorers
   - This tool expects the raw transaction ID bytes

3. **Witness Data**:
   - Only used if `--segwit` flag is present
   - Can be empty array `[]` for SegWit transactions without witness
   - Usually contains signature followed by public key

4. **Testing**: Use Bitcoin testnet transaction data to verify
   - Download test transactions from testnet explorers
   - Extract inputs, outputs, and witness data
   - Build JSON and test serialization

## Troubleshooting

### "Hex string '...' has odd length"
- Check that all hex values have even number of characters
- Each byte needs exactly 2 hex digits

### "Invalid hex in '...'"
- Verify hex string contains only 0-9 and a-f characters
- Remove any spaces or formatting from hex values

### "Input N missing 'prev_txid'"
- Check JSON structure: inputs must be an array of objects
- Each input object must have: prev_txid, vout, script_sig (or empty string)

### "Output N missing 'value'"
- Outputs must have: value (number in satoshis) and script_pubkey (hex string)

## Output Interpretation

The program outputs:
1. **Serialized Hex**: The complete transaction in hexadecimal format
2. **Transaction Size**: In bytes (useful for fee calculation)
3. **Details**: Version, SegWit status, input/output counts, locktime

Use the serialized hex to:
- Broadcast via Bitcoin RPC or Web API
- Calculate transaction size for fee estimation
- Verify with blockchain explorers

## Advanced: Building from Shell Script

```bash
#!/bin/bash

TX_VERSION="2"
TX_SEGWIT="--segwit"
TX_LOCKTIME="0"

# Build inputs
TX_INPUTS='[{
  "prev_txid": "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821",
  "vout": 1,
  "script_sig": "",
  "witness": ["3045022100...", "029cbb1e..."]
}]'

# Build outputs
TX_OUTPUTS='[{
  "value": 50000,
  "script_pubkey": "0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"
}]'

# Serialize
./target/release/serialize-trx \
  --version $TX_VERSION \
  $TX_SEGWIT \
  --locktime $TX_LOCKTIME \
  --inputs "$TX_INPUTS" \
  --outputs "$TX_OUTPUTS"
```
