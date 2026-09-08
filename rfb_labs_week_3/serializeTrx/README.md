# serializeTrx

A command-line tool for constructing and serializing Bitcoin transactions.  
No source-code edits required — supply all transaction data through flags.

---

## Build

```bash
cargo build --release
```

The binary is placed at `target/release/serializetrx`.

---

## Usage

```
serializetrx [OPTIONS] --input <INPUT>... --output <OUTPUT>...
```

### Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--version <N>` | Transaction version (`i32`) | `2` |
| `--segwit` | Enable SegWit serialization (adds marker/flag; serializes witness stacks) | off |
| `--input <INPUT>` | One transaction input (repeat for multiple) | required |
| `--output <OUTPUT>` | One transaction output (repeat for multiple) | required |
| `--witness <WITNESS>` | Witness stack for one input (repeat in same order as `--input`) | none |
| `--locktime <N>` | Locktime (`u32`) | `0` |

### Argument formats

#### `--input TXID:VOUT:SEQUENCE:SCRIPTSIG_HEX`

| Field | Description |
|-------|-------------|
| `TXID` | 64 hex characters (32 bytes). Automatically reversed into internal byte order. |
| `VOUT` | Decimal output index (`u32`). |
| `SEQUENCE` | Decimal or `0x`-prefixed hex (`u32`). Common values: `4294967295` / `0xffffffff`. |
| `SCRIPTSIG_HEX` | Hex-encoded scriptSig. Use an empty string `""` for native SegWit inputs. |

#### `--output VALUE_SATS:SCRIPTPUBKEY_HEX`

| Field | Description |
|-------|-------------|
| `VALUE_SATS` | Output value in satoshis (`u64`). |
| `SCRIPTPUBKEY_HEX` | Hex-encoded scriptPubKey. |

#### `--witness ITEM1_HEX,ITEM2_HEX,...`

- Provide **one `--witness` per input**, in the same order as the `--input` flags.
- Comma-separate multiple witness items within a single flag.
- Use an empty string `""` for inputs that carry no witness data.
- If no `--witness` flags are given at all, every input gets an empty witness stack.

---

## Validation

The program rejects invalid input before attempting serialization:

- Hex fields must contain only valid hexadecimal characters and have even length.
- TXID must be exactly 64 characters (32 bytes).
- VOUT and VALUE must be valid unsigned integers.
- When `--witness` flags are provided, their count must equal the number of `--input` flags.

Error messages identify the offending flag and field:

```
Error: --input[0]: input TXID error: TXID must be 64 hex characters (32 bytes), got 8 characters: "deadbeef"
Error: --witness[0]: witness item error: invalid hex character at position 0: "ZZ"
Error: --witness count (1) must match --input count (2). …
```

---

## Examples

### 1 — Native SegWit (P2WPKH) transaction

Reproduces the transaction that was previously hardcoded in the source.

```bash
serializetrx \
  --version 2 \
  --segwit \
  --input "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1:4294967295:" \
  --witness "3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301,029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358" \
  --output "69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b" \
  --output "29442:00149831122b93d21715c70db626ccc844d3c21f9687" \
  --locktime 0
```

Output:
```
Serialized hex:
0200000000010121c80d2b05c1106360a36e435ad8e418e85d0eb708d9f2bf216476b37bd0b08f0100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000

Transaction size: 223 bytes
```

---

### 2 — Legacy (non-SegWit) P2PKH transaction

```bash
serializetrx \
  --version 1 \
  --input "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2:0:4294967295:76a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88ac" \
  --output "50000:76a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88ac" \
  --locktime 0
```

Output:
```
Serialized hex:
0100000001b2a1f6e5d4c3b2a1f6e5d4c3b2a1f6e5d4c3b2a1f6e5d4c3b2a1f6e5d4c3b2a1000000001976a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88acffffffff0150c30000000000001976a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88ac00000000

Transaction size: 110 bytes
```

---

### 3 — Multi-input SegWit transaction (two inputs, one with witness)

When mixing SegWit and legacy inputs, supply an empty `--witness ""` for the input with no witness data:

```bash
serializetrx \
  --version 2 \
  --segwit \
  --input "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:0:4294967295:76a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88ac" \
  --input "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb:1:4294967295:" \
  --witness "" \
  --witness "3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301,029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358" \
  --output "80000:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b" \
  --locktime 500000
```

---

### 4 — Validation error examples

**Invalid hex character in witness:**
```bash
serializetrx --version 2 --segwit \
  --input "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1:0xffffffff:" \
  --witness "ZZZZ,029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358" \
  --output "69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"
# Error: --witness[0]: witness item error: invalid hex character at position 0: "ZZ"
```

**TXID too short:**
```bash
serializetrx --version 2 \
  --input "deadbeef:0:0xffffffff:" \
  --output "10000:76a914000000000000000000000000000000000000000088ac"
# Error: --input[0]: input TXID error: TXID must be 64 hex characters (32 bytes), got 8 characters: "deadbeef"
```

**Witness count mismatch:**
```bash
serializetrx --version 2 --segwit \
  --input "aaaa…:0:0xffffffff:" \
  --input "bbbb…:1:0xffffffff:" \
  --witness "aabb" \
  --output "50000:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"
# Error: --witness count (1) must match --input count (2). …
```

---

## Serialization format reference

```
SegWit transaction layout (BIP-141):
┌──────────────────────────────┐
│ Version          4 bytes     │
├──────────────────────────────┤
│ Marker           1 byte      │  0x00
│ Flag             1 byte      │  0x01
├──────────────────────────────┤
│ Input count      VarInt      │
│ Inputs           Variable    │
├──────────────────────────────┤
│ Output count     VarInt      │
│ Outputs          Variable    │
├──────────────────────────────┤
│ Witness          Variable    │  one stack per input
├──────────────────────────────┤
│ Locktime         4 bytes     │
└──────────────────────────────┘

Legacy transaction layout omits the Marker, Flag, and Witness fields.
```
