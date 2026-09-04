# Week 3 — Understanding Bitcoin Data

A from-scratch raw Bitcoin **transaction parser and decoder** in Rust, with a
`decoderawtransaction`-style JSON view and a small CLI.

No Bitcoin library is used for the wire format. The parser implements the classic
pre-SegWit layout and BIP144 (SegWit marker/flag/witness) directly, byte by byte.
The only dependencies are `clap` (CLI), `serde`/`serde_json` (JSON output), `hex`,
and `sha2` (for the double-SHA256 that produces the txid).

## Layout

| File | Responsibility |
|---|---|
| `src/reader.rs` | `ByteReader` cursor — little-endian integers and minimal CompactSize (VarInt) decoding, plus the CompactSize encoder. |
| `src/transaction.rs` | `Transaction`/`TxIn`/`TxOut`/`OutPoint` model, the parser, the serializer (legacy and witness), size/weight/vsize, and `txid`/`wtxid`. |
| `src/script.rs` | Script disassembly to `asm` and output-type classification (`pubkeyhash`, `scripthash`, `witness_v0_keyhash`, `witness_v1_taproot`, `multisig`, `nulldata`, …). |
| `src/decode.rs` | `DecodedTransaction` — the JSON projection shaped like Bitcoin Core's `decoderawtransaction`. |
| `src/lib.rs` | `parse_transaction_hex` and `decode_transaction_hex` entry points. |
| `src/main.rs` | `txdecode` CLI. |
| `tests/decode.rs` | Integration tests over legacy, SegWit, and coinbase vectors + error cases. |

## Running

```bash
# Decode from an argument (pretty JSON on stdout)
cargo run --quiet -- <raw-tx-hex>

# From a file or stdin
cargo run --quiet -- --file tx.hex
echo <raw-tx-hex> | cargo run --quiet -- --stdin

# Just the identifiers
cargo run --quiet -- --txid <raw-tx-hex>

# Compact single-line JSON
cargo run --quiet -- --compact <raw-tx-hex>

cargo test
cargo fmt --check
cargo clippy --all-targets
```

### Example

```bash
cargo run --quiet -- 0100000001c997a5e56e104102fa209c6a852dd90660a20b2d9c352423edce25857fcd3704000000004847304402204e45e16932b8af514961a1d3a1a25fdf3f4f7732e9d624c6c61548ab5fb8cd410220181522ec8eca07de4860a4acdd12909d831cc56cbbac4622082221a8768d1d0901ffffffff0200ca9a3b00000000434104ae1a62fe09c5f51b13905f07f06b99a2f7159b2225f374cd378d71302fa28414e7aab37397f554a7df5f142c21c1b7303b8a0626f1baded5c72a704f7e6cd84cac00286bee0000000043410411db93e1dcdb8a016b49840f8c53bc1eb68a382e97b1482ecad7b148a6909a5cb2e0eaddfb84ccf9744464f82e160bfa9b8b64f9d4c03f999b8643f656b412a3ac00000000
```

produces (abridged):

```json
{
  "txid": "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
  "hash": "f4184fc596403b9d638783cf57adfe4c75c605f6356fbc91338530e9831e9e16",
  "version": 1,
  "size": 275,
  "vsize": 275,
  "weight": 1100,
  "locktime": 0,
  "segwit": false,
  "vin": [ { "txid": "0437cd7f…97c9", "vout": 0, "scriptSig": { "asm": "3044…01", "hex": "47…01" }, "sequence": 4294967295 } ],
  "vout": [
    { "value_sat": 1000000000, "value_btc": "10.00000000", "n": 0, "scriptPubKey": { "asm": "04ae…4c OP_CHECKSIG", "hex": "41…ac", "type": "pubkey" } },
    { "value_sat": 4000000000, "value_btc": "40.00000000", "n": 1, "scriptPubKey": { "…": "…" } }
  ]
}
```

## What the decoder does

1. **Version** — 4 bytes, little-endian.
2. **SegWit detection** — if the byte where the input count belongs is `0x00`, it
   is the BIP144 marker; the next byte must be the flag `0x01`. The real input
   count follows. A non-`0x01` flag, or a marker with zero real inputs, is an
   error.
3. **Inputs** — for each: 32-byte previous txid (kept in internal byte order),
   4-byte vout, CompactSize-prefixed scriptSig, 4-byte sequence.
4. **Outputs** — for each: 8-byte value (satoshis), CompactSize-prefixed
   scriptPubKey.
5. **Witness** — only when SegWit: for each input, a CompactSize item count then
   CompactSize-prefixed items. A legacy input inside a SegWit tx has an empty
   stack (`0x00`).
6. **Locktime** — 4 bytes, little-endian.
7. **Identifiers** — `txid` = `SHA256d(serialization without marker/flag/witness)`,
   displayed in reverse byte order. `hash` (`wtxid`) = `SHA256d(full
   serialization)`; equal to the txid when there is no witness data.
8. **Sizes** — `size` is the full serialized length; `weight = base_size*3 +
   total_size` (BIP141); `vsize = ceil(weight/4)`.

## Notes and limitations

- **`asm`** is a readable disassembly: opcodes by name, data pushes as hex. It is
  deliberately *not* byte-identical to Bitcoin Core's rendering, which prints
  small pushes as decimal numbers.
- **Amounts** are given both as `value_sat` (exact `u64`) and `value_btc` (a fixed
  8-decimal string built by integer arithmetic — no floating point).
- The decoder does **not** derive addresses from scriptPubKeys (that needs
  Base58/Bech32 encoders, out of scope here); it reports the script `type`
  instead.
- CompactSize decoding rejects non-minimal encodings, matching Bitcoin Core.
- The parser is strict about trailing bytes: a valid decode consumes the whole
  input.

## Test vectors

| Vector | Description | Source |
|---|---|---|
| `LEGACY` | Block 170 payment, Satoshi → Hal Finney (pre-SegWit) | well-known |
| `SEGWIT` | Two inputs (one legacy, one native P2WPKH), witness stacks | BIP143-derived |
| `COINBASE` | Bitcoin genesis coinbase | well-known |

All expected `txid`/`wtxid` values in the tests were cross-checked against an
independent SHA256d implementation.
