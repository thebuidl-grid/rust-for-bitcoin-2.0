# decodetrx — Bitcoin Transaction Decoder (Rust)

A command-line tool that parses a raw Bitcoin (SegWit) transaction hex string and outputs its decoded fields as JSON — version, TXID, inputs, outputs, and locktime.

## What it does

Given a raw transaction hex string, `decodetrx`:
- Parses the transaction byte-by-byte according to Bitcoin's wire format
- Reads all CompactSize (VarInt) length prefixes
- Extracts every input (previous TXID, output index, scriptSig, sequence)
- Extracts every output (amount in BTC, scriptPubKey)
- Skips over SegWit witness data correctly (excluded from TXID by design)
- Computes the transaction's TXID via double-SHA256 of the non-witness (legacy) serialization
- Prints the result as pretty-printed JSON

## Wire format

```
┌──────────────────────────────┐
│ Version          4 bytes     │
├──────────────────────────────┤
│ Marker           1 byte      │
│ Flag             1 byte      │
├──────────────────────────────┤
│ Input count      VarInt      │
│ Inputs           Variable    │
├──────────────────────────────┤
│ Output count     VarInt      │
│ Outputs          Variable    │
├──────────────────────────────┤
│ Witness          Variable    │
├──────────────────────────────┤
│ Locktime         4 bytes     │
└──────────────────────────────┘
```

Bitcoin uses **little-endian** encoding for numeric fields (version, output index, sequence, amount, locktime) — least significant byte first.

This implementation assumes every transaction is SegWit-formatted (marker `0x00`, flag `0x01` always present) and returns an error if that marker/flag pair isn't found.

## Running it

```bash
cargo run -- <raw_transaction_hex>
```

Example:
```bash
cargo run -- 0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000
```

Output:
```json
{
  "transaction_id": "be9ea29072566edbc6827e3d9caf1d8c0b57cb0d5e74b95c721c46b3124cbe0b",
  "version": 2,
  "inputs": [
    {
      "txid": "bb16396a6af433052e23a09946322429ba2dd17f2809c978adc186c9047c2796",
      "output_index": 1,
      "script_sig": "",
      "sequence": 4294967295
    }
  ],
  "outputs": [
    {
      "amount": 1e-6,
      "script_pubkey": "0014274ae586ad2035efb4c25049c155f98310d7e106"
    },
    {
      "amount": 0.04462282,
      "script_pubkey": "0014599bcef6387256c6b019030c421b4a4d382fe260"
    }
  ],
  "lock_time": 0
}
```

This input has one native SegWit (P2WPKH) input, so `script_sig` is empty — the actual signature data lives in the witness stack, which is parsed (to keep byte-cursor alignment correct) but not included in the output, since the `Input` struct doesn't carry witness fields.

## Design notes

- **CompactSize (VarInt) parsing** (`read_compact_size`) handles all four Bitcoin encoding forms: a single byte for values ≤ 0xfc, and `0xfd`/`0xfe`/`0xff` prefixes for 2/4/8-byte little-endian values respectively.
- **TXID computation** hashes only the "legacy" serialization of the transaction — version, input count, inputs, output count, outputs, and locktime — explicitly excluding the SegWit marker/flag and witness data, per BIP-141. This is why the byte ranges actually hashed are collected separately (`legacy_bytes`) rather than hashing the raw input as-is.
- **TXID display** reverses the byte order before hex-encoding. Bitcoin's internal/wire representation of a TXID is stored in one byte order, but conventionally displayed (block explorers, `bitcoin-cli`) in the reverse — this matches that convention.
- **Errors, not panics**: every parsing function that can fail on malformed or truncated input returns `Result`, rather than panicking. Confirmed by a set of tests that feed empty, truncated, and non-hex input into the decoder and assert it returns `Err` cleanly.

## Testing

```bash
cargo test
```

16 tests covering:
- Correct decoding of a known, manually-verified real transaction (version, input fields, both output amounts, locktime, TXID format)
- Expected-failure handling: empty input, odd-length hex, non-hex characters, truncated data, and a missing/invalid SegWit marker
- Unit-level correctness of `read_compact_size` (all three multi-byte prefix forms) and `read_u32`, including cursor-advancement checks

## Tech

- Rust, `clap` for the CLI, `serde`/`serde_json` for output formatting, `sha2` for the double-SHA256 TXID hash.