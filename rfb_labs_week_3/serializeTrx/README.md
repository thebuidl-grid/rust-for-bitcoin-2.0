# serializetrx

Builds a Bitcoin transaction from values passed on the command line and prints
the serialised hex.

The transaction used to be written into `main.rs`, so changing any part of it
meant editing the source and rebuilding. The version, SegWit status, inputs,
outputs, witness data and locktime are all arguments now.

The serialisation code itself is unchanged. Only the source of the values moved.

```
┌──────────────────────────────┐
│ Version          4 bytes     │
├──────────────────────────────┤
│ Marker           1 byte      │  SegWit only (0x00)
│ Flag             1 byte      │  SegWit only (0x01)
├──────────────────────────────┤
│ Input count      VarInt      │
│ Inputs           Variable    │
├──────────────────────────────┤
│ Output count     VarInt      │
│ Outputs          Variable    │
├──────────────────────────────┤
│ Witness          Variable    │  SegWit only
├──────────────────────────────┤
│ Locktime         4 bytes     │
└──────────────────────────────┘
```

## Running it

```bash
cd rfb_labs_week_3/serializeTrx

cargo run -- --help                 # arguments and spec formats
cargo run -- --input <SPEC> --output <SPEC> [options]
cargo test                          # 31 tests
```

An installed binary is called `serializetrx`; through Cargo, everything after
`--` is passed to the program.

## Arguments

| Argument | Meaning | Default |
| --- | --- | --- |
| `--version <N>` | Transaction version. Also `--tx-version` | `2` |
| `-i, --input <SPEC>` | One input. Repeat for more | required |
| `-o, --output <SPEC>` | One output. Repeat for more | required |
| `-w, --witness <INDEX:ITEMS>` | Witness stack for input `INDEX` | none |
| `-l, --locktime <N>` | Locktime | `0` |
| `--segwit` / `--no-segwit` | Force a serialisation format | follows the data |
| `--txid-order <display\|internal>` | Byte order of the txid you typed | `display` |
| `-v, --verbose` | Also print a field by field breakdown | off |
| `--bytes` | Also print the decimal byte array | off |

### Spec formats

```
--input    <txid>:<vout>
           txid=<64 hex>,vout=<n>[,script_sig=<hex>][,sequence=<n>][,witness=<hex>|<hex>]
           mixed:  <txid>:<vout>,sequence=0xfffffffd,witness=<hex>|<hex>

--output   <satoshis>:<script_pubkey hex>
           amount=<satoshis>,script_pubkey=<hex>

--witness  <input index>:<hex>|<hex>          indexes start at 0
```

* Numbers may be decimal or `0x` hexadecimal, and may contain `_` separators:
  `sequence=4294967293`, `sequence=0xfffffffd` and `vout=1_000` all work.
* Witness items are separated by `|`. An empty value (`witness=`, `--witness 2:`)
  means an empty stack, which is what an input that needs no witness has. To put
  an empty item first, the way a P2WSH multisig stack starts, write
  `|<sig>|<script>`.
* Amounts are in satoshis. `script_sig` and `script_pubkey` are the scripts
  themselves; the program writes their length prefixes.

## Design decisions

The assignment left a few things open, so here is what I picked.

A txid is stored inside a transaction in the reverse of the order block
explorers print it in. Typing it the way you read it off an explorer seemed
more useful than typing it in internal order, so the program does the reversing.
`--txid-order internal` turns that off.

If neither `--segwit` nor `--no-segwit` is given, the format is taken from the
data. Witness items present means the BIP144 marker, flag and witness section,
none means legacy. The flags force the choice, and are rejected when they
disagree with the witness data. BIP144 says a transaction whose witness stacks
are all empty should use the legacy format, so `--segwit` with no witness items
is an error.

A witness belongs to an input, so it can go inside `--input` as `witness=`,
where it cannot end up on the wrong input. `--witness <index>:...` is there for
long stacks that would make the input unreadable. Giving both for the same input
is an error.

In a SegWit transaction every input has a witness stack, so an input left
without one still writes its zero count. Example 4 depends on this.

## Validation

Values are checked before anything is serialised, and the message says which
argument is wrong.

| Check | Example message |
| --- | --- |
| Hex digits | ``--input #1 txid: `g` is not a hexadecimal digit (position 63)`` |
| Even hex length | `--output #1 script_pubkey: hexadecimal value has an odd length (43 characters)` |
| No `0x` on byte strings | ``--output #1 script_pubkey: hexadecimal value must not start with `0x` `` |
| Txid width | `--input #1 txid: a txid must be 32 bytes, got 4` |
| Numbers | ``--output #1 amount: `one hundred` is not a whole number`` |
| Field widths | `--input #1 vout: 4294967296 is out of range (limit 0 to 4294967295 (0xffffffff))` |
| Amounts | `--output #1 amount: 2100000000000001 is out of range (limit 0 to 2100000000000000 satoshis, the whole supply)` |
| Spec shape | ``--input #1: cannot read `not-an-outpoint` `` |
| Keys | ``--input #1: unknown key `fee`, expected one of txid, vout, script_sig, sequence, witness`` |
| Duplicates | ``--input #1: `vout` was given more than once`` |
| Missing keys | ``--input #1: missing required key `txid` `` |
| Witness target | `--witness 1: there is no input 1, the transaction has 1` |
| Witness given twice | `--witness 0: input 0 already has a witness` |
| Format vs. data | `--segwit was given, but no witness items were supplied` |

Errors go to stderr with a hint and exit code 1, so stdout stays clean if you
pipe the hex somewhere:

```console
$ cargo run -- --input bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c279g:1 \
    --output 100:0014274ae586ad2035efb4c25049c155f98310d7e106
error: --input #1 txid: `g` is not a hexadecimal digit (position 63)
hint:  hexadecimal digits are 0-9, a-f and A-F

$ cargo run -- --segwit --input bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:1 \
    --output 100:6a
error: --segwit was given, but no witness items were supplied
hint:  BIP144 keeps the legacy format for a transaction whose witness stacks are all empty: drop --segwit, or add witness items
```

## Examples

### 1. A P2WPKH spend with one input and two outputs

The transaction that used to be hardcoded, passed in as arguments instead. Its
txid was written in internal order, so `--txid-order internal` is needed here.

```bash
cargo run -- \
  --version 2 \
  --txid-order internal \
  --input txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=1,witness=3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301\|029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358 \
  --output 69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b \
  --output 29442:00149831122b93d21715c70db626ccc844d3c21f9687 \
  --locktime 0
```

```
Serialized transaction (hex):
020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000

Transaction size: 223 bytes
```

### 2. An on-chain transaction, with the witness given by index

This one uses the txid in explorer order, the `txid:vout` shorthand and a
separate `--witness`, with `--verbose` to read the fields back. The output
matches mainnet transaction
`be9ea29072566edbc6827e3d9caf1d8c0b57cb0d5e74b95c721c46b3124cbe0b`.

```bash
cargo run -- --verbose \
  --input bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:1 \
  --witness 0:304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c201\|020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f1 \
  --output 100:0014274ae586ad2035efb4c25049c155f98310d7e106 \
  --output 4462282:0014599bcef6387256c6b019030c421b4a4d382fe260
```

```
Transaction
  version          2
  format           SegWit (BIP144 marker and flag, witness section)
  locktime         0  (no locktime)

Inputs (1)
  #0
    outpoint       bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:1
    script_sig     empty
    sequence       0xffffffff  (final)
    witness        2 items
                   [0]  71 bytes  304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c201
                   [1]  33 bytes  020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f1

Outputs (2)
  #0
    amount         100 sat  (0.00000100 BTC)
    script_pubkey  22 bytes, P2WPKH
                   0014274ae586ad2035efb4c25049c155f98310d7e106
  #1
    amount         4,462,282 sat  (0.04462282 BTC)
    script_pubkey  22 bytes, P2WPKH
                   0014599bcef6387256c6b019030c421b4a4d382fe260

  total out        4,462,382 sat  (0.04462382 BTC)

Size
  base             113 bytes
  total            222 bytes
  weight           561 WU
  virtual size     141 vB

Serialized transaction (hex):
0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000

Transaction size: 222 bytes
```

### 3. A legacy transaction with no witness

Block 170, Satoshi to Hal Finney. Version 1, with a scriptSig instead of a
witness. Neither format flag is passed, and since there are no witness items
the program picks legacy, so no marker and no flag are written.

```bash
cargo run -- \
  --version 1 \
  --input txid=0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9,vout=0,script_sig=47304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901 \
  --output 1000000000:4104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac \
  --output 4000000000:410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac
```

```
Serialized transaction (hex):
0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000

Transaction size: 275 bytes
```

### 4. Three inputs, mixed witness stacks, a block-height locktime

Two of the inputs have witnesses. The third has a scriptSig and no witness at
all, and its empty stack still has to be written. One output is a small P2WPKH
payment, the other is an `OP_RETURN` holding the whole supply, which is large
enough to need the 8 byte CompactSize.

```bash
cargo run -- \
  --version 2 \
  --txid-order internal \
  --input 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f:1,sequence=0xfffffffd \
  --input 202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f:0xfe,sequence=0xfffffffd \
  --input abababababababababababababababababababababababababababababababab:7,sequence=0xfffffffd,script_sig=471111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111 \
  --witness 0:303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030\|020202020202020202020202020202020202020202020202020202020202020202 \
  --witness 1:51515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151 \
  --output 100000:0014cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd \
  --output 2100000000000000:6a \
  --locktime 500000
```

```
Serialized transaction (hex):
02000000000103000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f0100000000fdffffff202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3ffe00000000fdffffffabababababababababababababababababababababababababababababababab0700000048471111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111fdffffff02a086010000000000160014cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd0040075af0750700016a0248303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030303030210202020202020202020202020202020202020202020202020202020202020202020140515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151515151510020a10700

Transaction size: 423 bytes
```

## Checking the output

Examples 2 and 3 are real transactions, and example 1 is what this program
printed before the refactor. Any of the hex above can be pasted back into the
decoder in [`../decodetrx`](../decodetrx) to read the fields:

```bash
cd ../decodetrx
cargo run -- <hex from above>
```

## Layout

| File | Contents |
| --- | --- |
| [`src/main.rs`](src/main.rs) | Reads the arguments, prints the result or the error |
| [`src/cli.rs`](src/cli.rs) | The argument definitions, and the parsing and validation behind them |
| [`src/transaction.rs`](src/transaction.rs) | The transaction model, the serialiser and CompactSize |
| [`src/hex.rs`](src/hex.rs) | Validated hex to bytes, and back |
| [`src/report.rs`](src/report.rs) | The printed output, including `--verbose` |
| [`src/error.rs`](src/error.rs) | Every validation failure, and its hint |
| [`tests/serialize.rs`](tests/serialize.rs) | Runs the binary and compares against known transactions |
