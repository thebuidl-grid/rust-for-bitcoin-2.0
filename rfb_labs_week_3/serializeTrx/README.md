# Bitcoin Transaction Serializer

A Rust command-line application that constructs and serializes Bitcoin
transactions from user-provided command-line arguments.

The transaction data is not hardcoded in the source code. Users can provide
the transaction version, SegWit status, inputs, outputs, witness data, and
locktime through the CLI.

## Features

- Serialize Bitcoin transactions into raw hexadecimal
- Support SegWit and non-SegWit transactions
- Support multiple inputs
- Support multiple outputs
- Support witness data
- Encode Bitcoin CompactSize / VarInt values
- Validate hexadecimal input
- Validate transaction fields
- Display the serialized transaction
- Display the serialized transaction size in bytes

## Requirements

- Rust
- Cargo

## Running the program

From this directory:

```bash
cargo run -- \
  --version 2 \
  --segwit true \
  --input "TXID:VOUT:SCRIPTSIG:SEQUENCE:WITNESS1|WITNESS2" \
  --output "VALUE:SCRIPTPUBKEY" \
  --output "VALUE:SCRIPTPUBKEY" \
  --locktime 0
```

### Input format

Each input is represented as:

```text
TXID:VOUT:SCRIPTSIG:SEQUENCE:WITNESS1|WITNESS2
```

Where:

- `TXID` is the previous transaction ID in hexadecimal
- `VOUT` is the previous output index
- `SCRIPTSIG` is the input script in hexadecimal
- `SEQUENCE` is the sequence number
- Witness items are separated by `|`

For an input without a scriptSig, leave that field empty.

For example:

```text
TXID:1::4294967295:WITNESS1|WITNESS2
```

### Output format

Each output is represented as:

```text
VALUE:SCRIPTPUBKEY
```

Where:

- `VALUE` is the amount in satoshis
- `SCRIPTPUBKEY` is the output script in hexadecimal

Multiple `--output` arguments can be provided to create multiple outputs.

## Example

The following command creates a SegWit transaction with one input and two
outputs:

```bash
cargo run -- \
  --version 2 \
  --segwit true \
  --input "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1::4294967295:3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301|029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358" \
  --output "69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b" \
  --output "29442:00149831122b93d21715c70db626ccc844d3c21f9687" \
  --locktime 0
```

Example output:

```text
Serialized transaction:
020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000

Transaction size: 223 bytes
```

## Validation

The program validates user-provided values before serialization.

For hexadecimal values:

- The input must contain an even number of characters.
- Each pair of characters must represent a valid hexadecimal byte.

Invalid values result in a meaningful error instead of producing an invalid
transaction.

## Bitcoin transaction serialization

The serializer follows the Bitcoin transaction structure:

```text
Version
    ↓
SegWit marker + flag (if SegWit)
    ↓
Input count (CompactSize)
    ↓
Inputs
    ↓
Output count (CompactSize)
    ↓
Outputs
    ↓
Witness data (if SegWit)
    ↓
Locktime
```

Numeric fields are serialized using little-endian encoding where required.

CompactSize / VarInt encoding is used for:

- Input count
- Output count
- ScriptSig length
- ScriptPubKey length
- Witness item count
- Witness item length

## Development checks

Run the following commands before submitting:

```bash
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

All checks should pass before creating the pull request.