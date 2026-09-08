# serializeTrx

Builds and serializes a Bitcoin transaction from data supplied though
command-line arguments.

## Building

```bash
cargo build --release
```

## Running


```bash
cargo run -- [OPTIONS]
```

One can also enter **interactive mode**, which prompts for
each value step by step :

```bash
cargo run
```

```
Transaction version: 2
Locktime: 0
Is this a SegWit transaction? (y/n): y
Input as prev_txid:vout:sequence:script_sig (blank to finish): 8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1:4294967295:
Input as prev_txid:vout:sequence:script_sig (blank to finish):
Output as value:script_pubkey (blank to finish): 69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b
Output as value:script_pubkey (blank to finish): 29442:00149831122b93d21715c70db626ccc844d3c21f9687
Output as value:script_pubkey (blank to finish):
Witness as input_index:item1,item2,... (blank to finish): 0:3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301,029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358
Witness as input_index:item1,item2,... (blank to finish):
```

Interactive mode produces identical output to the equivalent flag-based
 — inputs, outputs, and witness entries are each
terminated by an empty line, and invalid entries are rejected with a message
so one can retry that field without restarting the whole session.

Any invocation that passes at least one argument uses the flag-based mode.


### Options

| Flag         | Description                                                                 | Required | Repeatable |
|--------------|-------------------------------------------------------------------------------|----------|------------|
| `--version`  | Transaction version (i32). Default: `2`                                       | no       | no         |
| `--locktime` | Locktime (u32). Default: `0`                                                  | no       | no         |
| `--segwit`   | Mark this as a SegWit transaction (adds marker/flag and a witness section)    | no       | no         |
| `--input`    | `<prev_txid_hex>:<vout>:<sequence>:<script_sig_hex>` (script_sig may be empty)| yes      | yes        |
| `--output`   | `<value_sats>:<script_pubkey_hex>`                                           | yes      | yes        |
| `--witness`  | `<input_index>:<hex_item_1>,<hex_item_2>,...` — one per input needing witness data | no       | yes        |

All hex fields are validated before being converted to bytes — an odd-length
or non-hex string produces a clear error instead of a panic.

## Examples

### A SegWit P2WPKH transaction, one input, two outputs

```bash
cargo run -- \
  --version 2 --segwit --locktime 0 \
  --input "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1:4294967295:" \
  --witness "0:3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301,029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358" \
  --output "69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b" \
  --output "29442:00149831122b93d21715c70db626ccc844d3c21f9687"
```

Output:
```
Serialized Hex transaction:
020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000

Transaction size: 223 bytes
```

### A non-SegWit transaction, two inputs, one output

```bash
cargo run -- \
  --version 2 --locktime 0 \
  --input "aaaa000000000000000000000000000000000000000000000000000000aa:0:4294967295:" \
  --input "bbbb000000000000000000000000000000000000000000000000000000bb:1:4294967295:" \
  --output "10000:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"
```

Output:
```
Serialized Hex transaction:
0200000002aaaa000000000000000000000000000000000000000000000000000000aa0000000000ffffffffbbbb000000000000000000000000000000000000000000000000000000bb0100000000ffffffff011027000000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b00000000

Transaction size: 119 bytes
```

Note: no marker/flag bytes and no witness section, since `--segwit` was not
passed.

### Invalid input is rejected cleanly

```bash
cargo run -- \
  --version 2 --locktime 0 \
  --input "aaaa:0" \
  --output "10000:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"
```

Output:
```
Error: ParseError("--input 'aaaa:0' must have 4 colon-separated fields: prev_txid:vout:sequence:script_sig")
```

## Design notes

- Serialization logic (`serialize_transaction`, `encode_varint`) is unchanged
  from the original hardcoded version of previous assignment — only the data source changed.
- Repeatable flags (`--input`, `--output`, `--witness`) use a compact
  colon/comma, to keep the interface manageable for transactions with several inputs/outputs.
- `--witness` is matched to its input by index rather than by position in the
  argument list, so witness data can be supplied in any order or omitted for
  inputs that don't need it.

  - Running with no arguments drops into an interactive prompt mode that walks
  through each field and reuses the same `parse_input`/`parse_output`/
  `parse_witness` validation as the flag-based mode, so both paths accept
  identical data and produce identical output.