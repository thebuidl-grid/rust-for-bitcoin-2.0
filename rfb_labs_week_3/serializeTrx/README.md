# serialize-trx

A command-line tool that constructs and serializes a Bitcoin transaction
(legacy or SegWit) from user-supplied data. This is a refactor of a program
that originally had its transaction data (txid, amounts, scripts, witness)
hardcoded in `main.rs`; every value is now supplied via CLI flags, validated,
and turned into a correctly serialized raw transaction.

The wire-format serialization logic itself (version, marker/flag, inputs,
outputs, witness stack, locktime, CompactSize/"varint" length encoding) is
unchanged from the original implementation — only how the data gets into the
program has changed.

## Building

```sh
cd rfb_labs_week_3/serializeTrx
cargo build --release
```

The binary is `serialize-trx` (also runnable via `cargo run --`).

## Usage

```
serialize-trx [--tx-version <i32>] [--segwit] [--locktime <u32>]
              --input <INPUT> [--input <INPUT> ...]
              --output <OUTPUT> [--output <OUTPUT> ...]
```

| Flag | Required | Default | Description |
|---|---|---|---|
| `--tx-version` | no | `2` | Transaction `nVersion`. |
| `--segwit` | no | off (legacy) | Emit the BIP-144 marker (`0x00`) + flag (`0x01`) and serialize witness stacks. |
| `--locktime` | no | `0` | Transaction locktime. |
| `--input` | **yes**, repeatable | — | One transaction input. See format below. |
| `--output` | **yes**, repeatable | — | One transaction output. See format below. |

At least one `--input` and one `--output` must be provided; the program will
refuse to run without them.

### `--input` format

Comma-separated `key=value` fields:

```
txid=<64-char hex>,vout=<u32>[,sequence=<u32>][,script_sig=<hex>][,witness=<hex>|<hex>|...]
```

- `txid` (required): the previous output's txid as it should appear in the
  serialized bytes, hex-encoded. Must decode to exactly 32 bytes.
- `vout` (required): index of the previous output being spent.
- `sequence` (optional): defaults to `4294967295` (`0xffffffff`).
- `script_sig` (optional): hex-encoded scriptSig. Defaults to empty (typical
  for a SegWit input).
- `witness` (optional): pipe (`|`) separated list of hex-encoded witness
  stack items, e.g. `witness=<signature-hex>|<pubkey-hex>`. Only serialized
  when `--segwit` is passed; if witness data is given without `--segwit`, the
  program prints a warning and drops it, since the legacy format has no place
  to put it.

### `--output` format

```
value=<u64 satoshis>,script_pubkey=<hex>
```

Both fields are required.

## Validation

All hex fields are checked before decoding: an odd-length string or a
non-hex character produces a specific error identifying the field, the bad
value, and the reason. `txid` is additionally checked to decode to exactly 32
bytes. Integer fields report the offending value if they fail to parse.
Missing or unrecognized fields inside an `--input`/`--output` value are
rejected by name. Omitting `--input` or `--output` entirely is rejected by
the CLI parser itself.

## Output

The program prints the serialized transaction as hex and its size in bytes:

```
Serialized transaction (hex):
<hex>
Transaction size: <n> bytes
```

## Examples

### 1. SegWit transaction, one input, two outputs

```sh
cargo run --release -- \
  --tx-version 2 \
  --segwit \
  --locktime 0 \
  --input "txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=1,sequence=4294967295,witness=3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301|029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358" \
  --output "value=69886,script_pubkey=0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b" \
  --output "value=29442,script_pubkey=00149831122b93d21715c70db626ccc844d3c21f9687"
```

Output:

```
Serialized transaction (hex):
020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000
Transaction size: 223 bytes
```

### 2. Legacy transaction, two inputs, two outputs, no witness data

```sh
cargo run --release -- \
  --tx-version 1 \
  --locktime 500000 \
  --input "txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=0,script_sig=76a914,sequence=4294967293" \
  --input "txid=9fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc822,vout=2" \
  --output "value=100000,script_pubkey=76a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88ac" \
  --output "value=5000,script_pubkey=76a91489abcdefabbaabbaabbaabbaabbaabbaabbaabbc88ac"
```

Output:

```
Serialized transaction (hex):
01000000028fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821000000000376a914fdffffff9fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8220200000000ffffffff02a0860100000000001976a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88ac88130000000000001976a91489abcdefabbaabbaabbaabbaabbaabbaabbaabbc88ac20a10700
Transaction size: 163 bytes
```

### 3. Validation error: invalid hex

```sh
cargo run --release -- \
  --input "txid=zz,vout=0" \
  --output "value=1,script_pubkey=aa"
```

Output:

```
error: invalid value 'txid=zz,vout=0' for '--input <INPUTS>': --input: field 'txid' has invalid hex value 'zz': invalid hex character 'z' at position 0
```

### 4. Validation error: odd-length hex

```sh
cargo run --release -- \
  --input "txid=abc,vout=0" \
  --output "value=1,script_pubkey=aa"
```

Output:

```
error: invalid value 'txid=abc,vout=0' for '--input <INPUTS>': --input: field 'txid' has invalid hex value 'abc': hex string has an odd number of characters
```

### 5. Validation error: missing required argument

```sh
cargo run --release -- --output "value=1,script_pubkey=aa"
```

Output:

```
error: the following required arguments were not provided:
  --input <INPUTS>
```

## Testing

```sh
cargo test
```

Covers hex/argument validation (unit tests in `src/cli.rs`) and end-to-end
CLI behavior including a known-vector serialization check, legacy vs. SegWit
output, and error paths (integration tests in `tests/cli.rs`).
