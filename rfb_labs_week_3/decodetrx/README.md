# Bitcoin Transaction Decoder

A Rust command-line application that decodes a raw Bitcoin transaction
hex string into a readable JSON representation.

The decoder parses the raw transaction bytes and extracts the transaction
version, inputs, outputs, locktime, and transaction ID. It also handles
SegWit transactions and CompactSize/VarInt encoded values.

## How It Works

A raw Bitcoin transaction is a sequence of bytes. The decoder reads those
bytes sequentially and converts them into Rust structures.

The general transaction structure is:

```text
┌──────────────────────────────┐
│ Version          4 bytes     │
├──────────────────────────────┤
│ Marker           1 byte      │
│ Flag             1 byte      │
├──────────────────────────────┤
│ Input count      CompactSize │
│ Inputs           Variable    │
├──────────────────────────────┤
│ Output count     CompactSize │
│ Outputs          Variable    │
├──────────────────────────────┤
│ Witness          Variable    │
├──────────────────────────────┤
│ Locktime         4 bytes     │
└──────────────────────────────┘
```

For non-SegWit transactions, the marker, flag, and witness data are absent.

## Running the Decoder

From the `decodetrx` directory:

```bash
cargo run -- <RAW_TRANSACTION_HEX>
```

For example:

```bash
cargo run -- "0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000"
```

## Example Output

The decoder produces JSON containing the decoded transaction:

```json
{
  "transaction_id": "2a9241e605bca28b6347e57b1c25d2ce1581753f27f3552b26dd073658664b5e",
  "version": 2,
  "inputs": [
    {
      "txid": "96277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb",
      "output_index": 1,
      "script_sig": [],
      "sequence": 4294967295
    }
  ],
  "outputs": [
    {
      "amount": 1e-6,
      "script_pubkey": [
        0,
        20,
        39,
        74,
        229,
        134,
        173,
        32,
        53,
        239,
        180,
        194,
        80,
        73,
        193,
        85,
        249,
        131,
        16,
        215,
        225,
        6
      ]
    },
    {
      "amount": 0.04462282,
      "script_pubkey": [
        0,
        20,
        89,
        155,
        206,
        246,
        56,
        114,
        86,
        198,
        176,
        25,
        3,
        12,
        66,
        27,
        74,
        77,
        56,
        47,
        226,
        96
      ]
    }
  ],
  "lock_time": 0
}
```

## Fields Decoded

### Transaction Version

The first four bytes contain the transaction version.

Bitcoin uses little-endian encoding for numeric fields, so the bytes are
converted from their little-endian representation into a Rust `u32`.

For example:

```text
02 00 00 00
```

represents:

```text
2
```

### SegWit Marker and Flag

SegWit transactions contain:

```text
00 01
```

after the version.

The decoder checks for this marker and flag before parsing the inputs.

If SegWit is detected, the decoder skips these two bytes and later parses
the witness data.

### CompactSize / VarInt

Bitcoin uses CompactSize integers to represent variable-length values such
as:

- number of inputs
- number of outputs
- script lengths
- number of witness items
- witness item lengths

The encoding is:

```text
Value                     Encoding

0 - 252                   1 byte

253 - 65,535              FD + 2 bytes

65,536 - 4,294,967,295    FE + 4 bytes

larger values             FF + 8 bytes
```

The decoder reads the first byte and determines how many additional bytes
must be consumed.

### Inputs

Each transaction input contains:

```text
Previous transaction ID
Previous output index
ScriptSig length
ScriptSig
Sequence
```

The decoder reads each field sequentially and stores the result in an
`Input` struct.

### Outputs

Each transaction output contains:

```text
Amount
ScriptPubKey length
ScriptPubKey
```

The amount is stored internally in satoshis as a `u64`.

For JSON serialization, the `Amount` type converts satoshis into BTC.

For example:

```text
100,000,000 satoshis = 1 BTC
```

### Witness Data

For SegWit transactions, witness data appears after the outputs and before
the locktime.

Each input can contain multiple witness items.

The decoder reads:

```text
Number of witness items
Witness item length
Witness item bytes
```

The current `Transaction` model does not store the witness data. The decoder
still parses and consumes it so that the byte slice is positioned correctly
when reading the locktime.

### Locktime

The final four bytes of the transaction contain the locktime.

The decoder reads these bytes as a little-endian `u32`.

### Transaction ID

The transaction ID is calculated using double SHA-256:

```text
SHA256(SHA256(raw_transaction))
```

The resulting 32 bytes are reversed for conventional Bitcoin TXID display.

## Error Handling

The decoder validates the transaction as it reads it.

Examples of errors include:

- invalid hexadecimal input
- odd-length hexadecimal strings
- transactions that are too short
- insufficient bytes for a TXID
- insufficient bytes for a CompactSize value
- insufficient bytes for scripts
- insufficient bytes for witness data
- invalid script sizes
- unexpected bytes after locktime

Errors are returned using Rust's `Result` type rather than panicking.

## Main Rust Types

The decoded transaction is represented using several Rust structures.

```rust
pub struct Transaction {
    pub transaction_id: Txid,
    pub version: u32,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub lock_time: u32,
}
```

An input is represented as:

```rust
pub struct Input {
    pub txid: Txid,
    pub output_index: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
}
```

An output is represented as:

```rust
pub struct Output {
    pub amount: Amount,
    pub script_pubkey: Vec<u8>,
}
```

## Important Rust Concepts Used

This project demonstrates several Rust concepts:

- slices such as `&[u8]`
- mutable references such as `&mut &[u8]`
- structs
- `Vec<T>`
- `Result<T, E>`
- error propagation with `?`
- ownership and borrowing
- little-endian byte conversion
- SHA-256 hashing
- custom `Serialize` implementations
- Serde serialization
- command-line arguments
- parsing binary data

## Dependencies

The project uses:

- `serde`
- `serde_json`
- `sha2`

These libraries are used for JSON serialization and transaction ID
calculation.

## Example Command

```bash
cargo run -- "0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000"
```

The command outputs the decoded transaction as formatted JSON.