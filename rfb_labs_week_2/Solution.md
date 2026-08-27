## Parts 1–2 — Data Model Review

### The provided types

- **`OutputType`** — an enum listing the different Bitcoin output script types this model
  supports: `P2pkh`, `P2wpkh`, `P2tr`, and `OpReturn`. It's a closed set of possibilities,
  which is exactly what an enum is for.
- **`TxOutput`** — a struct representing one transaction output: how much value it carries
  (`value: u64`, in satoshis), who it pays (`recipient: String`), and what kind of script
  locks it (`output_type: OutputType`).
- **`OutPoint`** — a struct identifying one specific previous output by its transaction ID
  and output index (`txid: String`, `vout: u32`). This is the coordinate system for "which
  exact coin is being spent."
- **`InputKind`** — an enum with two variants, `Regular` and `Coinbase` (details below).
- **`Transaction`** — a struct holding everything together: `version`, a `Vec<InputKind>`,
  a `Vec<TxOutput>`, and `locktime`.

### Why `InputKind` is an enum

A transaction input is fundamentally one of two mutually exclusive shapes:

- **`Regular { previous_output: OutPoint, value: u64, sequence: u32 }`** — spends an
  existing UTXO. It needs to say *which* output it's spending (`previous_output`), how
  much that output was worth (`value`), and a sequence number.
- **`Coinbase { block_height: u32, reward: u64 }`** — the one special input that appears
  only as the first input of a block's first transaction, creating new coins out of
  nothing. It has no previous output to point to at all — instead it carries the block
  height and the reward amount.

These two shapes don't share fields and are never a "little bit of both": a real input is
never simultaneously spending a previous output *and* minting a block reward. An enum
expresses "exactly one of these variants, and nothing else is possible" at the type level.
A single struct with optional fields for everything (`previous_output: Option<OutPoint>`,
`block_height: Option<u32>`, etc.) could represent the same data, but it would also allow
nonsensical states the compiler couldn't rule out — e.g. an input with both a
`previous_output` and a `block_height` set, or neither. The enum makes invalid
combinations unrepresentable.

### How matching forces both variants to be handled

Every time this model needs to answer "how much value does this input represent" —
in `total_input_value`, the `BitcoinValue` implementation, and `Display` — it has to
`match` on the `InputKind`. Rust's `match` is exhaustive by default: if a match on an enum
doesn't cover every variant, the code fails to compile. Concretely, this means it's
structurally impossible to write a function that computes a total and silently forgets to
count the `Coinbase` case (or the `Regular` case) — the compiler rejects the code at build
time rather than letting a missing case surface later as an incorrect balance or a runtime
panic. This is the core safety benefit of modelling inputs as an enum rather than, say, a
struct with a `kind: String` field and manual `if`/`else` branching, where forgetting a
case would compile fine and fail silently.
