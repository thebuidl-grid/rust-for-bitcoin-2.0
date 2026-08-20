# serializetrx

Construct and serialize a Bitcoin transaction entirely from command-line arguments.
No transaction data is hardcoded. The program validates all inputs, serializes the
transaction according to the Bitcoin wire format, and prints the result as hex.

Both legacy and SegWit transactions are supported.

---

## Build

```bash
cargo build --release
```

The binary is placed at `target/release/serializetrx`.

For development you can use `cargo run --` followed by the arguments shown below.

---

## Usage

```
serializetrx [OPTIONS] --input <INPUT>... --output <OUTPUT>...

Options:
      --tx-version <TX_VERSION>  Transaction version [default: 2]
      --segwit                   Enable SegWit serialization (marker/flag + witness)
      --input <INPUT>            Input: TXID_HEX:VOUT:SEQUENCE_HEX:SCRIPTSIG_HEX
      --output <OUTPUT>          Output: VALUE_SATS:SCRIPTPUBKEY_HEX
      --witness <WITNESS>        Witness stack for one input: ITEM1_HEX,ITEM2_HEX,...
      --locktime <LOCKTIME>      Locktime [default: 0]
  -h, --help                     Print help
  -V, --version                  Print version
```

### Argument formats

**`--input TXID_HEX:VOUT:SEQUENCE_HEX:SCRIPTSIG_HEX`**

| Field | Description |
|---|---|
| `TXID_HEX` | 64 hex chars (32 bytes). Use the display-order TXID from a block explorer. |
| `VOUT` | Previous output index as a decimal integer. |
| `SEQUENCE_HEX` | 4-byte sequence as hex, e.g. `ffffffff`. |
| `SCRIPTSIG_HEX` | Hex-encoded scriptSig. Empty for native SegWit inputs. |

**`--output VALUE_SATS:SCRIPTPUBKEY_HEX`**

| Field | Description |
|---|---|
| `VALUE_SATS` | Amount in satoshis as a decimal integer. |
| `SCRIPTPUBKEY_HEX` | Hex-encoded scriptPubKey. |

**`--witness ITEM1_HEX,ITEM2_HEX,...`**

One `--witness` flag per input, in the same order as `--input`.
Items are comma-separated hex strings.
Use `--witness ""` for an input with no witness data.
Required when `--segwit` is set and any witness data exists.

---

## Examples

### Example 1: SegWit P2WPKH transaction

A version-2 SegWit transaction with one input, two outputs, and a witness stack
containing a DER signature and a compressed public key.

```bash
cargo run -- \
  --tx-version 2 \
  --segwit \
  --input "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1:ffffffff:" \
  --output "69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b" \
  --output "29442:00149831122b93d21715c70db626ccc844d3c21f9687" \
  --witness "3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301,029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358" \
  --locktime 0
```

Output:

```
Serialized hex:
0200000000010121c80d2b05c1106360a36e435ad8e418e85d0eb708d9f2bf216476b37bd0b08f0100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000

Transaction size: 223 bytes
```

---

### Example 2: Legacy P2PKH transaction

A version-1 legacy transaction with one input (scriptSig present) and one output.
No `--segwit` flag, no `--witness` flags.

```bash
cargo run -- \
  --tx-version 1 \
  --input "e6b6f79c0c8c31e09edb0c4498f4f27e2b1e08e42f0e5fe77dc38e8b1fa5ee1e:0:ffffffff:76a914a457d0c7e37d7dc2e3e5c0c6e6f1de803cd0ccb888ac" \
  --output "50000:76a914f4c3da0e7a0a32e47dbdb5ce8f7dc65a5bc27d5888ac" \
  --locktime 0
```

Output:

```
Serialized hex:
01000000011eeea51f8b8ec37de75f0e2fe4081e2b7ef2f498440cdb9ee0318c0c9cf7b6e6000000001976a914a457d0c7e37d7dc2e3e5c0c6e6f1de803cd0ccb888acffffffff0150c30000000000001976a914f4c3da0e7a0a32e47dbdb5ce8f7dc65a5bc27d5888ac00000000

Transaction size: 110 bytes
```

---

## Validation errors

The program validates all hex fields before serializing and exits with a descriptive
error on any invalid input.

**Invalid hex character:**

```bash
cargo run -- --tx-version 2 --segwit \
  --input "ZZZZ...:0:ffffffff:" \
  --output "1000:0014abc" --witness ""
# Error: input[0].txid: invalid hex '...': Invalid character 'Z' at position 0
```

**Odd-length hex string:**

```bash
# sequence "fffff" has 5 characters (not a whole number of bytes)
# Error: input[0].sequence: hex string has odd length (5); each byte needs two hex characters
```

**Witness count does not match input count:**

```bash
# 2 inputs but only 1 --witness flag
# Error: --segwit requires one --witness entry per input (2 inputs, 1 witness flags supplied)
```

**`--witness` without `--segwit`:**

```bash
# Error: --witness flags supplied but --segwit was not set
```
