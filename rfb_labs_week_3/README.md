# Week 3 — Understanding Bitcoin Data

Decoding raw Bitcoin transactions from bytes: version, CompactSize counts,
inputs, outputs, amounts, TXIDs, and SegWit witness data.

## Layout

```
rfb_labs_week_3/
├── decodetrx/      # Decoder + Clap CLI → JSON
│   └── src/{lib.rs, transaction.rs, main.rs}
├── trxparse/       # Cursor-based parser → JSON object
│   └── src/{lib.rs, main.rs}
├── serializeTrx/   # The inverse: struct → raw bytes
│   └── src/main.rs
└── tests/{decodetrx.rs, trxparse.rs}
```

`decodetrx` and `trxparse` decode the same bytes two different ways — typed
structs with derived serde on one side, a `Cursor` and hand-built JSON on the
other. `tests/trxparse.rs` asserts they agree, so each is a check on the other.
`serializeTrx` runs the pipeline backwards, and its output feeds straight back
into `decodetrx`.

## Running

```bash
cargo test

# Decode a transaction
cargo run -p decodetrx -- 0100000001c997a5e5...

# Parse one (JSON object form); reads stdin, or falls back to a sample
cargo run -p trxparse
echo "0200000000010196277c04..." | cargo run -p trxparse

# Serialize a hard-coded transaction back to raw hex
cargo run -p serializetrx
```

## Transaction layout

```
┌──────────────────────────────┐
│ Version          4 bytes     │
├──────────────────────────────┤
│ Marker           1 byte      │  SegWit only (0x00)
│ Flag             1 byte      │  SegWit only (0x01)
├──────────────────────────────┤
│ Input count      CompactSize │
│ Inputs           Variable    │
├──────────────────────────────┤
│ Output count     CompactSize │
│ Outputs          Variable    │
├──────────────────────────────┤
│ Witness          Variable    │  SegWit only
├──────────────────────────────┤
│ Locktime         4 bytes     │
└──────────────────────────────┘
```

## Written answers

### What is CompactSize and why does Bitcoin use it?

CompactSize (VarInt) encodes an integer in 1, 3, 5, or 9 bytes depending on how
large it is. The first byte is either the value itself or a marker for how many
bytes follow:

| First byte | Total width | Value read from       |
|------------|-------------|-----------------------|
| `0x00..fc` | 1 byte      | the first byte itself |
| `0xfd`     | 3 bytes     | next 2 bytes, LE      |
| `0xfe`     | 5 bytes     | next 4 bytes, LE      |
| `0xff`     | 9 bytes     | next 8 bytes, LE      |

It exists to save space. Counts in Bitcoin are almost always small — a
transaction usually has a handful of inputs and outputs — so spending 8 bytes on
a number that is nearly always under 253 would waste bytes in every transaction
ever made. CompactSize appears everywhere a variable-length field needs a
length: input count, output count, every script length, and every witness item.

### How does a decoder tell a legacy transaction from a SegWit one?

After the 4-byte version comes the input count. A valid transaction can never
have zero inputs, so a `0x00` in that position cannot be a real count — BIP144
uses it as the SegWit **marker**, followed by a **flag** byte of `0x01`. If the
byte is anything else, the transaction is legacy and that byte is the genuine
input count.

This means the decoder has to peek without consuming. `decodetrx` checks
`bytes.first()` and only advances past the two bytes when the marker is present;
`trxparse` records the cursor position and rewinds with `set_position` when the
byte turns out not to be a marker.

### Why is the SegWit TXID not just a hash of the raw bytes?

This is the part that is easy to get wrong. The TXID is the double-SHA256 of the
**legacy** serialization — version, inputs, outputs, locktime — with the marker,
flag, and all witness data stripped out. Hashing the raw SegWit bytes as they
arrived gives the **wtxid**, a different identifier.

That separation is the whole point of SegWit: witness data sits outside the
TXID, so changing a signature cannot change the transaction's identity. That
fixes transaction malleability, which is what made payment channels and
Lightning practical.

In `decode_transaction` this is done by recording two byte offsets while
parsing — where the inputs begin and where the outputs end — then rebuilding the
legacy serialization as `version ++ inputs ++ outputs ++ locktime` before
hashing.

### Why are TXIDs displayed reversed?

The 32 bytes on the wire are in internal byte order, but every explorer, RPC,
and block header display shows them reversed (big-endian). The convention is
historical, not principled, but a decoder that skips the reversal produces TXIDs
that match nothing anyone else prints. `Txid` keeps the wire bytes internally
and reverses only in its `Display` impl, so the reversal happens exactly once,
at the boundary.

### Why not use `f64` for amounts?

Amounts are read as `u64` satoshis and only converted to BTC for display.
Floating point cannot represent most decimal fractions exactly, so doing
arithmetic on BTC-denominated floats accumulates error — a real hazard when the
numbers are money.

Even display needs care: `100` satoshis is `0.000001` BTC, which `serde_json`
prints as `1e-6`. The `as_btc` serializer formats to a fixed 8 decimal places
and emits the digits as a raw JSON number, so the output reads `0.00000100` —
what `bitcoin-cli` shows.

### What breaks a naive parser?

Every one of these has a test in `tests/decodetrx.rs`:

- **Odd-length or non-hex input** — rejected at the `hex::decode` step.
- **Truncated transactions** — the scaffold's `unwrap()`-everywhere style
  panics; returning `io::Error` from each read turns this into a clean failure.
- **Trailing bytes** — if the cursor is not exactly at the end after locktime,
  the input was not a single valid transaction.
- **Invalid SegWit flag** — a `0x00` marker followed by anything other than
  `0x01`.
- **Oversized script lengths** — a corrupt CompactSize can claim a 4 GB script.
  Checking the claimed length against the bytes actually remaining *before*
  allocating turns a potential OOM into an error.

## Test vectors

| | Transaction | Source |
|---|---|---|
| Legacy | `f4184fc5…9e16` — block 170, Satoshi → Hal Finney (10 BTC + 40 BTC change) | mempool.space |
| SegWit | `be9ea290…be0b` — testnet P2WPKH (100 sats + 4,462,282 sats) | blockstream.info/testnet |

Expected TXIDs were read off the explorers before the assertions were written,
so these are genuine checks rather than snapshots of the decoder's own output.
