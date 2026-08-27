# Bitcoin Transaction Serializer

A Rust CLI application for constructing and serializing Bitcoin transactions without modifying the Rust source code.

## Features

- Transaction version
- Multiple inputs
- Multiple outputs
- SegWit status (--segwit enables SegWit; omit it to disable SegWit)
- SegWit witness data
- Multiple witness items
- Locktime
- Hexadecimal validation
- Bitcoin CompactSize serialization
- Serialized transaction hexadecimal output
- Transaction size in bytes
- Meaningful validation errors

## Requirements
- Rust
- Cargo

## Usage

```powershell
cargo run -- --help
```

### Input format

```text
TXID:VOUT:SEQUENCE:SCRIPTSIG
```

Repeat --input for multiple inputs.

### Output format

```text
AMOUNT_IN_SATOSHIS:SCRIPTPUBKEY
```

Repeat --output for multiple outputs.

### Witness format

```text
INPUT_INDEX:ITEM_HEX
```

Repeat --witness for multiple witness items.

## Serialization

Bitcoin transaction integers use little-endian encoding and variable-length fields use CompactSize encoding.

SegWit transactions include the marker, flag, and witness data.

## Validation

User-provided hexadecimal values are validated before conversion into bytes. Invalid values produce meaningful errors instead of panics.

## Output

The program displays:

- Transaction version
- SegWit status
- Transaction input count
- Transaction output count
- Witness data status
- Locktime
- Serialized transaction hexadecimal
- Transaction size in bytes

## Examples

### 1 input and 1 output

```powershell
cargo run -- `
  --version 2 `
  --input "1111111111111111111111111111111111111111111111111111111111111111:0:4294967295:" `
  --output "100000:0014aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" `
  --locktime 0
```

### Multiple inputs and outputs

```powershell
cargo run -- `
  --version 2 `
  --input "1111111111111111111111111111111111111111111111111111111111111111:0:4294967295:" `
  --input "2222222222222222222222222222222222222222222222222222222222222222:1:4294967295:" `
  --output "100000:0014aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" `
  --output "50000:0014bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" `
  --locktime 0
```

### SegWit with multiple witness items

```powershell
cargo run -- `
  --version 2 `
  --segwit `
  --input "1111111111111111111111111111111111111111111111111111111111111111:0:4294967295:" `
  --output "100000:0014aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" `
  --witness "0:0102030405060708090a" `
  --witness "0:02030405060708090a0b" `
  --locktime 0
```

### Invalid hexadecimal input

```powershell
cargo run -- `
  --version 2 `
  --input "NOT_HEX:0:4294967295:" `
  --output "100000:0014aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
```

The program returns a meaningful validation error instead of panicking.rnrn## Verification

Run:

```powershell
cargo fmt -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build
git diff --checkrn