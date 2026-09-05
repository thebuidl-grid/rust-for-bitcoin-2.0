# serializeTrx

Constructs and serializes a Bitcoin transaction (legacy or SegWit) entirely
from command-line arguments — no source-code edits needed to serialize a
different transaction. Prints the serialized transaction as hex and reports
its size in bytes.

## Build & run

```bash
cd rfb_labs_week_3/serializeTrx
cargo run -- [FLAGS]
```

## Flags

| Flag | Format | Required | Default |
|---|---|---|---|
| `--version` | integer | no | `2` |
| `--segwit` | (presence flag) | no | absent (legacy) |
| `--input` | `txid_hex:vout:sequence:script_sig_hex` | yes, at least one | — |
| `--output` | `value_sats:script_pubkey_hex` | yes, at least one | — |
| `--witness` | `input_index:item_hex` | no | — |
| `--locktime` | unsigned integer | no | `0` |

`--input` and `--output` may be repeated to build a transaction with multiple
inputs/outputs. `--witness` may also be repeated; `input_index` is 0-based and
refers to the position of an `--input` on the command line (the first
`--input` is index `0`, the second is index `1`, and so on). An input with no
scriptSig (typical for a SegWit input) still needs the trailing colon, e.g.
`txid:1:4294967295:` with nothing after the last `:`.

## Validation

- Every hex field (`txid`, `script_sig`, `script_pubkey`, witness items) is
  checked for valid hex (even length, hex digits only) before conversion to
  bytes.
- A txid must decode to exactly 32 bytes.
- `vout`, `sequence`, output `value`, and `locktime` must parse as their
  expected integer type.
- `--input`/`--output`/`--witness` must each have the correct number of
  colon-delimited fields.
- At least one `--input` and one `--output` are required.
- A `--witness`'s `input_index` must refer to an input that was actually
  provided.
- `--witness` without `--segwit` is rejected — only SegWit inputs carry
  witness data.

Any failure prints a specific message to stderr and exits with a non-zero
status, rather than producing an incorrect transaction or panicking.

## Examples

### A SegWit transaction: one input, two outputs, two witness items

```bash
cargo run -- \
  --version 2 \
  --segwit \
  --input 8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1:4294967295: \
  --output 69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b \
  --output 29442:00149831122b93d21715c70db626ccc844d3c21f9687 \
  --witness 0:3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301 \
  --witness 0:029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358 \
  --locktime 0
```

```
Serialized transaction (hex): 020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000
Transaction size: 223 bytes
```

### Multiple inputs
