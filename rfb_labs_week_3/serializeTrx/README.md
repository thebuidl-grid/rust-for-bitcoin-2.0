# serializetrx

Builds a Bitcoin transaction from command-line arguments and serializes it
to raw transaction bytes, printing the byte array, the hex encoding, and the
final size in bytes. Supports both legacy and SegWit transactions, and any
number of inputs, outputs, and witness items — no source changes needed to
serialize a different transaction.

## Running

```sh
cargo run -- [OPTIONS] --input <INPUT>... --output <OUTPUT>...
```

## Flags

| Flag | Required | Default | Format |
|---|---|---|---|
| `--version` | no | `2` | signed 32-bit integer (nVersion) |
| `--segwit` | no | `true` | `true` or `false` |
| `--locktime` | no | `0` | unsigned 32-bit integer (nLockTime) |
| `--input` | yes, repeatable | — | `PREV_TXID:VOUT:SEQUENCE[:SCRIPT_SIG_HEX]` |
| `--output` | yes, repeatable | — | `VALUE_SATS:SCRIPT_PUBKEY_HEX` |
| `--witness` | no, repeatable | — | `INPUT_INDEX:WITNESS_ITEM_HEX` |

Notes:

- `PREV_TXID` must be exactly 64 hex characters (32 bytes).
- `SEQUENCE` accepts a decimal number or a `0x`-prefixed hex number, e.g.
  `0xffffffff`.
- `SCRIPT_SIG_HEX` in `--input` is optional and defaults to an empty script
  (the usual case for a native SegWit input).
- All hex fields are validated before being converted into bytes: odd-length
  or non-hex-digit strings are rejected with a descriptive error naming the
  offending flag.
- `--witness INPUT_INDEX:HEX` attaches a witness item to the `--input` at
  that 0-based position, in the order the `--witness` flags are given. Pass
  it multiple times per input for a multi-item witness stack (e.g. signature
  + pubkey). Witness data is only serialized when `--segwit true`.

## Examples

### 1. Reproduce a real P2WPKH transaction (one input, two outputs)

```sh
cargo run -- \
  --version 2 --segwit true --locktime 0 \
  --input 8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1:0xffffffff \
  --output 69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b \
  --output 29442:00149831122b93d21715c70db626ccc844d3c21f9687 \
  --witness 0:3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301 \
  --witness 0:029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358
```

```
Serialized Hex transaction:
020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000

Transaction size: 223 bytes
```

### 2. Multiple inputs and outputs, witness data on more than one input

```sh
cargo run -- \
  --version 2 --locktime 0 \
  --input aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:0:0xffffffff \
  --input bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb:2:0xffffffff \
  --output 50000:0014aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
  --output 25000:0014bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb \
  --output 10000:0014cccccccccccccccccccccccccccccccccccccccc \
  --witness 0:aabb \
  --witness 0:ccdd \
  --witness 1:eeff
```

```
Transaction size: 198 bytes
```

### 3. Legacy (non-SegWit) transaction with an inline scriptSig

```sh
cargo run -- \
  --segwit false --version 1 --locktime 500000 \
  --input aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:0:0xffffffff:76a914aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa88ac \
  --output 100000:76a914bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb88ac
```

```
Serialized Hex transaction:
0100000001aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa000000001976a914aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa88acffffffff01a0860100000000001976a914bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb88ac20a10700

Transaction size: 110 bytes
```

No marker/flag bytes or witness section are written when `--segwit false`.

### 4. Validation errors

```sh
$ cargo run -- --input abcd:0:0 --output 100:1234
Error: invalid --input 'abcd:0:0': PREV_TXID must be 32 bytes (64 hex chars), got 2

$ cargo run -- --input zzzz:0:0 --output 100:1234
Error: invalid --input 'zzzz:0:0': bad PREV_TXID hex: invalid digit found in string

$ cargo run -- --input aa...aa:0:0 --output 100:1234 --witness 5:ab
Error: invalid --witness '5:ab': INPUT_INDEX 5 out of range, only 1 input(s) provided
```

## Structure

- `src/transaction.rs` — the `Transaction`/`TxInput`/`TxOutput` types and the
  serialization logic (`serialize_transaction`, `encode_varint`,
  `hex_to_bytes`, `bytes_to_hex`).
- `src/cli.rs` — the `clap`-based argument definitions plus parsing and
  validation of `--input` / `--output` / `--witness` specs into a
  `Transaction`.
- `src/main.rs` — wires the CLI parsing to the serializer and prints the
  result.
