# serializeTrx

Builds and serializes a Bitcoin transaction (legacy or SegWit) entirely from
command-line arguments — no transaction data is hardcoded in the source.
The core byte-level serialization logic is unchanged from the original
version; only how the transaction is *specified* has changed.

## Build

```bash
cargo build --release
```

## Usage

```bash
cargo run -- [OPTIONS] --input <SPEC> --output <SPEC>
```

| Flag | Meaning | Default |
|---|---|---|
| `--version <n>` | transaction version | `2` |
| `--locktime <n>` | transaction locktime | `0` |
| `--segwit` | present = SegWit transaction (adds marker/flag, serializes witness data); absent = legacy | absent (legacy) |
| `--input <SPEC>` | one input; repeat for multiple inputs | required, at least one |
| `--output <SPEC>` | one output; repeat for multiple outputs | required, at least one |
| `--witness <SPEC>` | one witness stack item for one input; repeat (in argument order) to build up a stack | optional |

`--input`, `--output` and `--witness` each take a small `key=value,key=value`
spec string, so one flag occurrence carries all the fields for that item:

- **`--input`**: `txid=<64 hex chars>,vout=<n>[,sequence=<n>][,script-sig=<hex>]`
  - `txid` — previous transaction's txid, 32 bytes of hex (required)
  - `vout` — output index being spent (required)
  - `sequence` — defaults to `4294967295`
  - `script-sig` — hex, defaults to empty (i.e. a SegWit input)
- **`--output`**: `value=<satoshis>,script-pubkey=<hex>`
- **`--witness`**: `index=<input index, 0-based>,item=<hex>`

Run `cargo run -- --help` for the full reference and an example.

### Validation

All hex fields (`txid`, `script-sig`, `script-pubkey`, witness `item`) are
checked for even length and valid hex digits before being converted to
bytes; `txid` is additionally checked to be exactly 32 bytes. Spec strings
are checked for unknown/duplicate/missing fields, and numeric fields are
checked to actually parse as numbers. `--witness` is checked to reference an
input index that exists. Any failure prints a specific, human-readable error
and exits non-zero — nothing is serialized on invalid input.

## Output

The program prints:

- the serialized transaction as a byte array (`Debug` format)
- the serialized transaction as hex
- the transaction size in bytes

## Examples

**SegWit P2WPKH transaction, one input, two outputs** (this is the same
transaction that used to be hardcoded in `main.rs`):

```bash
cargo run -- \
  --version 2 --locktime 0 --segwit \
  --input txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=1 \
  --output value=69886,script-pubkey=0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b \
  --output value=29442,script-pubkey=00149831122b93d21715c70db626ccc844d3c21f9687 \
  --witness index=0,item=3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301 \
  --witness index=0,item=029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358
```

```
Serialized Hex transaction:
020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000

Transaction size: 223 bytes
```

**Legacy transaction, two inputs, one output, custom sequence, no `--segwit`
flag and no witness data:**

```bash
cargo run -- \
  --version 1 --locktime 500000 \
  --input txid=000000000000000000000000000000000000000000000000000000000000000a,vout=0,sequence=4294967293,script-sig=76a914 \
  --input txid=000000000000000000000000000000000000000000000000000000000000000b,vout=2 \
  --output value=100000,script-pubkey=76a914000000000000000000000000000000000000000088ac
```

This omits the marker/flag bytes and the witness section entirely, since
`--segwit` was not passed.

**Invalid input is rejected with a specific error, e.g. a malformed txid:**

```bash
$ cargo run -- --input txid=zz,vout=0 --output value=1,script-pubkey=00
Error: "invalid input spec 'txid=zz,vout=0': field 'txid': hex string 'zz' contains a non-hexadecimal character"
```
