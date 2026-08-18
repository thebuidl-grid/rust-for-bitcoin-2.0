# serializeTrx

A command-line tool that builds and serializes a raw Bitcoin transaction from
flags you pass in — no transaction data is hardcoded in the source. It
supports multiple inputs, multiple outputs, and per-input witness data, and
prints the resulting transaction as hex plus its size in bytes.


## Usage

```
serializeTrx --version <i32> [--segwit] --input <spec> [--input <spec>]... \
             --output <spec> [--output <spec>]... [--witness <spec>]... \
             --locktime <u32>
```

| Flag | Required | Repeatable | Format |
|---|---|---|---|
| `--version <i32>` | yes | no | Transaction version, e.g. `2` |
| `--segwit` | no | no | Boolean presence flag — marks the tx as segwit and enables witness serialization |
| `--input <spec>` | yes (>=1) | yes | `txid_hex:vout[:sequence[:scriptsig_hex]]` |
| `--output <spec>` | yes (>=1) | yes | `value_sats:scriptpubkey_hex` |
| `--witness <spec>` | no | yes | `input_index:item_hex` |
| `--locktime <u32>` | yes | no | Locktime |
| `-h`, `--help` | — | — | Print usage and exit |


## Examples

### 1. Single input, two outputs, segwit (matches the original hardcoded example)

```bash
cargo run -- \
  --version 2 \
  --segwit \
  --input 8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1 \
  --witness 0:3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301 \
  --witness 0:029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358 \
  --output 69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b \
  --output 29442:00149831122b93d21715c70db626ccc844d3c21f9687 \
  --locktime 0
```

```
Serialized transaction (hex):
020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000

Transaction size: 223 bytes
```

### 2. Multiple inputs, multiple outputs, witness data on more than one input

```bash
cargo run -- \
  --version 2 --segwit \
  --input 1111111111111111111111111111111111111111111111111111111111111111:0 \
  --input 2222222222222222222222222222222222222222222222222222222222222222:1:4294967293 \
  --witness 0:3044022100aabbccdd \
  --witness 0:0233333333 \
  --witness 1:30440221001122 \
  --output 50000:0014aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
  --output 25000:0014bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb \
  --locktime 0
```

This produces a two-input, two-output transaction where input 0 gets two
witness items and input 1 gets one — the second `--input` also demonstrates
supplying an explicit `sequence` (`4294967293`, i.e. RBF-signaling).

### 3. Validation error: `--witness` without `--segwit`

```bash
cargo run -- \
  --version 2 \
  --input 1111111111111111111111111111111111111111111111111111111111111111:0 \
  --witness 0:0233333333 \
  --output 50000:0014aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
  --locktime 0
```

```
Error: --witness was given but --segwit was not set; add --segwit or remove the --witness flags
```
(exit code 1)

Other invalid input is rejected the same way, e.g.:

```bash
cargo run -- --version 2 --input 1111111111111111111111111111111111111111111111111111111111111111:0 --output 50000:0014a --locktime 0
# Error: invalid hex in --output scriptpubkey: '0014a' (Hex string must have even length, got 5 characters)
```
