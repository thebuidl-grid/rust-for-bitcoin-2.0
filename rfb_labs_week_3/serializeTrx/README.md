# Bitcoin Transaction Serializer CLI

A command-line tool written in Rust that allows you to construct, validate, and serialize Bitcoin transactions dynamically without modifying any source code. It supports multiple inputs (both legacy and SegWit), multiple outputs, Custom sequences, custom script signatures/PubKeys, and locktime settings.

## Installation & Compilation

Make sure you have Rust and Cargo installed. Then compile the binary using:

```bash
cargo build --bin serialize_trx
```

---

## Command-Line Arguments

The application accepts the following arguments:

- `-v, --version <VERSION>`: The transaction version (default: `2`).
- `-s, --segwit`: Flag indicating if this is a SegWit transaction. When specified, a SegWit marker (`0x00`) and flag (`0x01`) are included, and the witness data is serialized.
- `-l, --locktime <LOCKTIME>`: Transaction locktime (4-byte unsigned integer, default: `0`).
- `-i, --input <INPUT_SPEC>`: Transaction input specification. **This argument can be repeated multiple times.**
  - **Syntax**: `prev_txid=<hex>,vout=<u32>[,sequence=<u32>][,script_sig=<hex>][,witness=<hex1>:<hex2>:...]`
  - **Fields**:
    - `prev_txid` (Required): 32-byte (64 hex characters) transaction ID of the output being spent.
    - `vout` (Required): The output index in the previous transaction.
    - `sequence` (Optional): Input sequence number (defaults to `4294967295` / `0xffffffff`).
    - `script_sig` (Optional): Hexadecimal script signature (defaults to empty).
    - `witness` (Optional): Colon-separated (`:`) list of hexadecimal witness items (defaults to empty).
- `-o, --output <OUTPUT_SPEC>`: Transaction output specification. **This argument can be repeated multiple times.**
  - **Syntax**: `value=<u64>,script_pubkey=<hex>`
  - **Fields**:
    - `value` (Required): Output amount in Satoshis.
    - `script_pubkey` (Required): Hexadecimal script pubkey.

---

## Execution Examples

### 1. Recreating the Original SegWit Transaction

To serialize the original transaction that was hardcoded in `main.rs`:

- **Version**: 2
- **SegWit**: Enabled
- **Inputs**: 1 input
  - `prev_txid`: `8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821`
  - `vout`: 1
  - `sequence`: `0xffffffff`
  - `witness`: `3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301` and `029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358`
- **Outputs**: 2 outputs
  - Output 0: `value=69886`, `script_pubkey=0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b`
  - Output 1: `value=29442`, `script_pubkey=00149831122b93d21715c70db626ccc844d3c21f9687`

Run command:
```bash
cargo run --bin serialize_trx -- \
  --version 2 \
  --segwit \
  --locktime 0 \
  --input prev_txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=1,witness=3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301:029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358 \
  --output value=69886,script_pubkey=0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b \
  --output value=29442,script_pubkey=00149831122b93d21715c70db626ccc844d3c21f9687
```

---

### 2. Legacy Transaction (Non-SegWit)

A transaction without SegWit. The inputs do not contain witness data.

```bash
cargo run --bin serialize_trx -- \
  --version 1 \
  --locktime 0 \
  --input prev_txid=4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b,vout=0,script_sig=47304402201234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef02201234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef01 \
  --output value=5000000000,script_pubkey=76a914a2bf4f95880c5d6428e8a60e0a5c43d8ef439b1a88ac
```

---

## Validation & Errors

The tool validates all arguments dynamically and handles failures without panicking.

- **Odd-length or non-hex string input**:
  ```bash
  Error: Error in input 0: Invalid prev_txid: Hex string must have an even length
  ```
- **Invalid characters in hex strings**:
  ```bash
  Error: Error in output 0: Invalid script_pubkey: Invalid hex character sequence 'zz': Invalid character 'z' at position 0
  ```
- **Incorrect TXID length**:
  ```bash
  Error: Error in input 0: prev_txid must be exactly 32 bytes (64 hex characters), got 3 bytes
  ```
- **Witness specified without --segwit**:
  ```bash
  Error: Validation Error: Input 0 contains witness items but SegWit status (--segwit) is not enabled.
  ```
- **Invalid integers**:
  ```bash
  Error: Error in input 0: Invalid vout index 'abc':...
  ```
