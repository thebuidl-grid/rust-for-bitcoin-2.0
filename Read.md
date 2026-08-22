# Bitcoin Transaction Serializer (CLI)

Week 4 refactor of the Week 3 serializer
([PR #136](https://github.com/thebuidl-grid/rust-for-bitcoin-2.0/pull/136)).

The original program had a single hardcoded transaction baked into
`main()`. This version keeps the exact same serialization logic
(`serialize_transaction`, `encode_varint`, `bytes_to_hex`) but takes every
transaction field — version, SegWit status, inputs, outputs, witness data,
and locktime — from command-line arguments, so you can build and serialize
any transaction without touching the Rust source.

## Build

```bash
cargo build --release
```

The binary is produced at `target/release/btc_tx_serializer`.

## Usage

```
btc_tx_serializer [OPTIONS] --input <TXID:VOUT[:SEQUENCE[:SCRIPT_SIG]]> --output <VALUE:SCRIPT_PUBKEY>
```

| Flag         | Repeatable | Format                                    | Notes |
|--------------|:----------:|--------------------------------------------|-------|
| `--version`  | no  | integer, e.g. `1` or `2`                         | defaults to `2` |
| `--segwit`   | no  | flag, no value                                   | present = SegWit tx (adds marker/flag + witness section) |
| `--locktime` | no  | `u32`                                            | defaults to `0` |
| `--input`    | yes | `TXID:VOUT[:SEQUENCE[:SCRIPT_SIG]]`              | at least one required |
| `--output`   | yes | `VALUE:SCRIPT_PUBKEY`                            | at least one required |
| `--witness`  | yes | `INPUT_INDEX:HEX_ITEM_1,HEX_ITEM_2,...`          | optional, one flag per input that needs a witness stack |

### `--input` fields

- `TXID` — 64 hex characters (32-byte previous txid). Validated for length and hex-only characters.
- `VOUT` — previous output index (`u32`).
- `SEQUENCE` — optional, defaults to `0xffffffff` if omitted.
- `SCRIPT_SIG` — optional hex-encoded scriptSig, defaults to empty (typical for native SegWit inputs).

### `--output` fields

- `VALUE` — amount in satoshis (`u64`).
- `SCRIPT_PUBKEY` — hex-encoded output script.

### `--witness` fields

- `INPUT_INDEX` — zero-based index into the `--input` flags, in the order they were supplied.
- A comma-separated list of hex-encoded witness stack items for that input.
- Only used when `--segwit` is set; if `--witness` is omitted for a given input, that input serializes with an empty witness stack.

All hex values are validated (even length, hex-digits only) before conversion to bytes. Numeric fields are validated as proper unsigned integers of the correct width. Invalid input produces a specific error message on stderr and a non-zero exit code — nothing is guessed or silently defaulted beyond what's documented above.

## Output

The program prints:

- A short summary (version, segwit flag, input/output counts, locktime)
- The serialized transaction as a hex string
- The transaction size in bytes

## Examples

### 1. Single-input, two-output SegWit transaction

```bash
./target/release/btc_tx_serializer \
  --version 2 \
  --segwit \
  --locktime 0 \
  --input 8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1 \
  --output 69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b \
  --output 29442:00149831122b93d21715c70db626ccc844d3c21f9687 \
  --witness "0:3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301,029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358"
```

```
Transaction summary:
  version:  2
  segwit:   true
  inputs:   1
  outputs:  2
  locktime: 0

Serialized transaction (hex):
020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000

Transaction size: 223 bytes
```

This reproduces the transaction that used to be hardcoded in the original program — now built entirely from CLI flags.

### 2. Multiple inputs, multiple outputs, legacy (non-SegWit)

```bash
./target/release/btc_tx_serializer \
  --version 1 \
  --locktime 500000 \
  --input aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:0:4294967293:76a914000000000000000000000000000000000000000088ac \
  --input bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb:2 \
  --output 100000:76a914111111111111111111111111111111111111111188ac \
  --output 50000:76a914222222222222222222222222222222222222222288ac
```

```
Transaction summary:
  version:  1
  segwit:   false
  inputs:   2
  outputs:  2
  locktime: 500000

Serialized transaction (hex):
0100000002aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa000000001976a914000000000000000000000000000000000000000088acfdffffffbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb0200000000ffffffff02a0860100000000001976a914111111111111111111111111111111111111111188ac50c30000000000001976a914222222222222222222222222222222222222222288ac20a10700

Transaction size: 185 bytes
```

The first input supplies an explicit sequence and scriptSig (legacy-style spend); the second input uses the defaults (`sequence = 0xffffffff`, empty scriptSig).

### 3. Validation errors

Odd-length hex is rejected before conversion:

```bash
./target/release/btc_tx_serializer --input abcde:0 --output 1000:00
```
```
Error: txid 'abcde' must be exactly 64 hex characters (32 bytes), got 5
```

Non-hex characters are rejected:

```bash
./target/release/btc_tx_serializer \
  --input zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz:0 \
  --output 1000:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b
```
```
Error: invalid hex string 'zzzz...zzzz': hex string contains non-hexadecimal characters
```

Missing required flags are caught by the CLI parser itself:

```bash
./target/release/btc_tx_serializer --output 1000:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b
```
```
error: the following required arguments were not provided:
  --input <TXID:VOUT[:SEQUENCE[:SCRIPT_SIG]]>
```

A witness pointing at a non-existent input index is caught too:

```bash
./target/release/btc_tx_serializer \
  --segwit \
  --input 8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1 \
  --output 1000:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b \
  --witness "5:aa"
```
```
Error: witness input index 5 is out of range: only 1 input(s) were provided
```

## Design notes

- **Keeping the diff logical**: `TxInput`, `TxOutput`, `Transaction`, `hex_to_bytes`, `bytes_to_hex`, `serialize_transaction`, and `encode_varint` are unchanged in behavior from the original — only `hex_to_bytes` gained an explicit hex-character check (the original only checked length parity and relied on `from_str_radix` failing for bad chars, which loses the "validate before converting" intent from the requirements). Everything else that changed is CLI plumbing.
- **Compound flags over many flags**: inputs/outputs/witness data are each expressed as one colon/comma-delimited flag rather than 5+ separate flags per input, so `--input`/`--output`/`--witness` can simply be repeated for multiple entries without needing to track indices for unrelated flags.
- **Witness is addressed by input index** rather than positionally paired with `--input`, so witness data can be supplied in any order (or omitted) independent of input order.
- **Exit codes**: `0` on success, non-zero on any validation failure or CLI parse error (`clap`'s own errors, e.g. missing required flags, use its standard exit code `2`; this program's own validation errors use `1`).