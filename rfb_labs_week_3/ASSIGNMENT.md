# Week 3 — Bitcoin Transaction Decoder

Build a Bitcoin transaction decoder in Rust: parse transaction fields, inputs,
outputs, TXIDs, amounts, VarInt/CompactSize values, and SegWit transaction data.
Complete the decoder, build the CLI interface, and test against real
transactions.

## Requirements checklist

### Parsing primitives
- [x] `read_compact_size` — VarInt/CompactSize across all four widths (`0x00..fc`, `0xfd`, `0xfe`, `0xff`)
- [x] `read_u32` / `read_u64` — little-endian integer reads
- [x] `read_version_byte` — 4-byte version field
- [x] `read_amount` — 8-byte satoshi amount into `Amount`
- [x] `read_txid` — 32-byte TXID in wire byte order
- [x] `read_script_size` — CompactSize-prefixed script as hex
- [x] `hash_row_transaction` — double-SHA256

### Transaction structure
- [x] `Transaction`, `Input`, `Output`, `Amount`, `Txid` types
- [x] TXIDs displayed reversed (big-endian), matching explorers
- [x] Amounts converted sats → BTC via the `BitcoinValue` trait
- [x] Amounts formatted to 8 decimal places (no scientific notation)

### Decoding
- [x] Legacy transactions
- [x] SegWit (BIP144): marker `0x00`, flag `0x01`, per-input witness stacks
- [x] Correct SegWit TXID — hashed over the **legacy** serialization, with
      marker, flag, and witness stripped out
- [x] JSON output

### CLI
- [x] Clap CLI taking a raw transaction hex argument
- [x] `--help` / `--version`
- [x] Errors to stderr with a non-zero exit code

### Error handling
- [x] Invalid hex rejected
- [x] Truncated transactions rejected (no panic)
- [x] Trailing bytes after lock_time rejected
- [x] Invalid SegWit flag rejected
- [x] Oversized script lengths rejected before allocating

### Tests
- [x] Legacy decoding tests
- [x] SegWit decoding tests
- [x] TXIDs asserted against block-explorer values
- [x] `trxparse` and `decodetrx` cross-checked against each other
- [x] Malformed-input tests

## Test vectors

| | Transaction | Verified against |
|---|---|---|
| Legacy | `f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16` — block 170, Satoshi → Hal Finney | mempool.space |
| SegWit | `be9ea29072566edbc6827e3d9caf1d8c0b57cb0d5e74b95c721c46b3124cbe0b` — testnet P2WPKH | blockstream.info/testnet |

Both expected values were taken from block explorers before the assertions were
written, so the TXID tests are real checks rather than snapshots of this
decoder's own output.

## Status

18 tests passing, clean build with no warnings.

```
cargo test
cargo run -p decodetrx -- <raw_transaction_hex>
```
