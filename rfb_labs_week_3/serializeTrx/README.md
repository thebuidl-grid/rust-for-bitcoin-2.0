# serializetrx

Constructs and serializes a Bitcoin transaction into raw wire-format hex,
entirely from command-line arguments — no transaction data is hardcoded in
the source.

## Build

```
cargo build --release
```

## Usage

```
serializetrx [OPTIONS] --input <INPUT> --output <OUTPUT>
```

| Flag | Required | Description |
|---|---|---|
| `--version <i32>` | no (default `2`) | Transaction version |
| `--segwit` | no (default off) | Marks the transaction as SegWit: adds the marker/flag bytes and serializes witness data |
| `--input <INPUT>` | yes, repeatable | One transaction input (see format below) |
| `--output <OUTPUT>` | yes, repeatable | One transaction output (see format below) |
| `--locktime <u32>` | no (default `0`) | Transaction locktime |

Pass `--input` once per input and `--output` once per output — any number of
each is supported.

### `--input` format

```
prev_txid_hex:vout:script_sig_hex:sequence[:witness_item_hex,witness_item_hex,...]
```

- `prev_txid_hex` — 32 bytes of hex (64 characters), in internal (non-reversed)
  byte order, i.e. the same byte order it is written in on the wire.
- `vout` — output index in the previous transaction, decimal `u32`.
- `script_sig_hex` — scriptSig as hex. Use an empty string (`""` between the
  colons) for native SegWit inputs, which carry no scriptSig.
- `sequence` — decimal `u32` (use `4294967295` for the default max sequence,
  `0xffffffff`).
- `witness_item_hex,...` — optional. A comma-separated list of hex-encoded
  witness items for this input. Only used when `--segwit` is passed; if any
  input has witness items and `--segwit` is missing, the program errors out
  rather than silently dropping them.

### `--output` format

```
value_sats:script_pubkey_hex
```

- `value_sats` — amount in satoshis, decimal `u64`.
- `script_pubkey_hex` — scriptPubKey as hex.

### Validation

All hex fields are validated (even length, valid hex digits) before being
converted to bytes, `prev_txid` is checked to be exactly 32 bytes, and numeric
fields are checked to fit their expected integer type. Invalid input produces
a specific error message pointing at the bad argument and field rather than a
panic.

### Output

The program prints:

- The serialized transaction as a Rust byte-array (`Debug` of `Vec<u8>`)
- The serialized transaction as hex
- The transaction size in bytes

## Examples

### SegWit, single input/output pair (native P2WPKH)

```
cargo run -- \
  --version 2 --segwit \
  --input "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1::4294967295:3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301,029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358" \
  --output "69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b" \
  --output "29442:00149831122b93d21715c70db626ccc844d3c21f9687" \
  --locktime 0
```

```
Serialized Hex transaction:
020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000

Transaction size: 223 bytes
```

### Legacy (non-SegWit), multiple inputs and outputs

```
cargo run -- \
  --version 1 \
  --input "1111111111111111111111111111111111111111111111111111111111111111:0::4294967295" \
  --input "2222222222222222222222222222222222222222222222222222222222222222:1::4294967295" \
  --output "50000:76a914000000000000000000000000000000000000000088ac" \
  --output "25000:76a914111111111111111111111111111111111111111188ac" \
  --output "10000:76a914222222222222222222222222222222222222222288ac" \
  --locktime 500000
```

`--segwit` is simply omitted, so no marker/flag bytes or witness data are
serialized.

### Invalid input

```
cargo run -- --input "zz:0::0" --output "1000:00"
```

```
error: invalid value 'zz:0::0' for '--input <INPUTS>': invalid hex string 'zz': invalid digit found in string
```
