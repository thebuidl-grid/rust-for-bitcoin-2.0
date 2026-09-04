# serializetrx

Builds and serializes a Bitcoin transaction from values given on the command
line. Nothing about the transaction is hardcoded — version, SegWit status,
inputs, outputs, witness data, and locktime all come from arguments, so
different transactions can be built without touching the source.

Outputs the serialized transaction in hexadecimal and its size in bytes.

## Running

```bash
cargo run -- [OPTIONS] --input <SPEC> --output <SPEC>
cargo run -- --help
```

## Options

| Flag | Meaning | Default |
|---|---|---|
| `--version <N>` | Transaction version | `2` |
| `--segwit` | Add the BIP144 marker and flag | off |
| `--input <SPEC>` | An input; repeat for more | required |
| `--output <SPEC>` | An output; repeat for more | required |
| `--witness <SPEC>` | Witness stack for one input; repeatable | none |
| `--locktime <N>` | Transaction locktime | `0` |
| `--txid-order <ORDER>` | `display` or `internal` | `display` |

### Spec formats

```
--input    txid:vout[:script_sig_hex[:sequence]]
--output   amount_sats:script_pubkey_hex
--witness  input_index:item_hex[,item_hex...]
```

`script_sig` defaults to empty and `sequence` to `4294967295` (`0xffffffff`), so
the common case is just `--input <txid>:<vout>`. Numbers accept decimal or a
`0x` hex prefix, so `4294967295` and `0xffffffff` are equivalent.

### A note on txid byte order

Bitcoin stores txids on the wire in one byte order and every explorer displays
them in the opposite one. This program takes txids the way you read them off
mempool.space and reverses them for you. If you already have a value in wire
order, pass `--txid-order internal` to have it written as-is.

Getting this backwards is the single easiest way to build a transaction that
looks right and spends nothing, so it is worth being deliberate about.

## Examples

### 1. Minimal legacy transaction

One input, one output, everything else defaulted.

```bash
cargo run -- \
  --input 0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9:0 \
  --output 50000:0014274ae586ad2035efb4c25049c155f98310d7e106
```

```
Serialized transaction (hex):
0200000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd37040000000000ffffffff0150c3000000000000160014274ae586ad2035efb4c25049c155f98310d7e10600000000

Transaction size: 82 bytes
```

### 2. A real SegWit transaction

This rebuilds testnet transaction
[`be9ea290…be0b`](https://blockstream.info/testnet/tx/be9ea29072566edbc6827e3d9caf1d8c0b57cb0d5e74b95c721c46b3124cbe0b)
— a P2WPKH spend with two outputs and a two-item witness. The output matches
the raw transaction on-chain byte for byte.

```bash
cargo run -- --version 2 --segwit \
  --input bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796:1 \
  --output 100:0014274ae586ad2035efb4c25049c155f98310d7e106 \
  --output 4462282:0014599bcef6387256c6b019030c421b4a4d382fe260 \
  --witness 0:304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c201,020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f1
```

```
Serialized transaction (hex):
0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000

Transaction size: 222 bytes
```

Note the empty scriptSig in the input spec: for a native SegWit input the
signature and public key live in the witness instead.

### 3. A real legacy transaction, with a scriptSig

Block 170 — the Satoshi to Hal Finney transaction. Version 1, no SegWit, one
input carrying a full scriptSig, and two outputs (10 BTC out, 40 BTC change).
This also matches the on-chain bytes exactly.

```bash
cargo run -- --version 1 \
  --input 0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9:0:47304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901:0xffffffff \
  --output 1000000000:4104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac \
  --output 4000000000:410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac
```

```
Transaction size: 275 bytes
```

### 4. Multiple inputs and outputs

Three inputs, three outputs. Repeat the flag for each.

```bash
TXID=0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9
cargo run -- \
  --input $TXID:0 --input $TXID:1 --input $TXID:2 \
  --output 1000:0014274ae586ad2035efb4c25049c155f98310d7e106 \
  --output 2000:51 \
  --output 3000:6a
```

### 5. Multiple inputs where only one is witness-bearing

Witness data attaches to an input by index. Any SegWit input you leave out gets
an empty witness stack, which is what keeps the witness block aligned with the
input list.

```bash
TXID=0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9
cargo run -- --segwit \
  --input $TXID:0 --input $TXID:1 \
  --output 1000:51 \
  --witness 1:aabb
```

### 6. Non-zero locktime and a custom sequence

A sequence below `0xffffffff` is what makes locktime take effect.

```bash
cargo run -- \
  --input 0437cd7f8525ceed2324359c2d0ba26006d92d856a9c20fa0241106ee5a597c9:0::0xfffffffe \
  --output 50000:51 \
  --locktime 800000
```

## Validation

Values are checked before anything is serialized, and errors go to stderr with
exit code 1. Actual output:

```
$ cargo run -- --input abcd:0 --output 1000:51
error: txid must be exactly 32 bytes (64 hex characters), got 2 bytes

$ cargo run -- --input $TXID:0 --output 1000:zz
error: script_pubkey: 'zz' is not valid hexadecimal

$ cargo run -- --input $TXID:0:abc --output 1000:51
error: script_sig: hex must have an even number of characters, got 3

$ cargo run -- --input $TXID:0 --output 2100000000000001:51
error: output amount 2100000000000001 sats exceeds the 21 million BTC supply cap (2100000000000000 sats)

$ cargo run -- --input nope --output 1000:51
error: --input 'nope': expected txid:vout[:script_sig_hex[:sequence]]

$ cargo run -- --input $TXID:0 --output 1000:51 --witness 0:aabb
error: --witness was supplied without --segwit; witness data only exists in SegWit transactions

$ cargo run -- --segwit --input $TXID:0 --output 1000:51
error: --segwit was supplied but no --witness data; BIP144 forbids the marker and flag when there is no witness

$ cargo run -- --segwit --input $TXID:0 --output 1000:51 --witness 5:aabb
error: --witness refers to input 5 but only 1 input(s) were supplied
```

The last two are worth calling out. BIP144 says a transaction must not carry the
marker and flag if it has no witness data, so `--segwit` with no `--witness` is
rejected rather than quietly producing a transaction no node would accept. And
a witness index past the end of the input list is a typo, not a default.

## Design decisions

The assignment left the interface open. The choices made here:

- **Repeatable flags with colon-separated fields** rather than a config file,
  since the requirement was that values arrive as command-line arguments.
- **Witness attaches by input index** (`--witness 1:...`) rather than positionally,
  so a transaction where only some inputs are witness-bearing stays readable.
- **Amounts in satoshis**, not BTC, to avoid floating-point rounding on money.
- **txids in explorer order by default**, because that is where people copy them
  from, with `--txid-order internal` as the escape hatch.
- **Sensible defaults** for `script_sig`, `sequence`, `version`, and `locktime`,
  so the common case stays short.

The serialization logic itself is unchanged from the original program — the
refactor moved it into `lib.rs` and put argument parsing and validation in front
of it.

## Tests

```bash
cargo test
```

19 tests. The two most important rebuild the real transactions in examples 2 and
3 and assert the output equals the on-chain raw hex byte for byte. Those raw
transactions were taken from block explorers, so they are genuine checks and not
snapshots of this program's own output. The rest cover CompactSize boundaries,
byte-order handling, defaults, and every validation error above.
