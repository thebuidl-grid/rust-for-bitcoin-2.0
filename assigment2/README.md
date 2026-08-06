# Rust for Bitcoin 2.0 — Week 2

A simplified Bitcoin transaction model in Rust: structs and enums for
transactions/UTXOs, trait-based validation, `Result`-based error handling,
collections for UTXO management, and a `Wallet` type that ties ownership and
borrowing together into a working coin-selection and payment flow.

No starter scaffold (`ASSIGNMENT.md`, pre-existing `transaction.rs`/`error.rs`
with `TODO`s) shipped with this repo, so the crate below was designed from the
assignment brief rather than filled into an existing skeleton. All tests are
enabled by default (none are `#[ignore]`d).

## Layout

- `src/error.rs` — `TransactionError`, the single error type returned by every
  fallible operation in the crate.
- `src/transaction.rs` — `Address`, `OutPoint`, `TxOutput`, `TxInput`
  (a `Regular`/`Coinbase` enum), `TransactionStatus`, `Transaction`
  (built via `add_input`/`add_output`, with `total_input_value`/`fee`
  against a `UtxoSet`), the `Validate` trait, and the `BitcoinValue` trait.
- `src/utxo.rs` — `Utxo`, `UtxoSet` (backed by a `HashMap<OutPoint, TxOutput>`),
  `CoinSelectionStrategy`, and `Selection`.
- `src/wallet.rs` — `Wallet`, which owns a `UtxoSet` and builds transactions
  from it.
- `src/main.rs` — a runnable payment example.
- `tests/payment_flow.rs` — integration tests exercising the public API
  end-to-end.

## Design notes

**Error handling.** Every fallible operation returns
`Result<T, TransactionError>`. `TransactionError` is a single enum covering
structural validation failures (`EmptyInputs`, `EmptyOutputs`,
`ZeroValueOutput`, `DuplicateInput`), lookup failures (`UtxoNotFound`),
arithmetic (`AmountOverflow`, guarded with `checked_add` instead of `+`), and
`InsufficientFunds { required, available }` for failed coin selection. It
implements `Display` and `std::error::Error` so callers can `?` it through or
report it with `{err}`.

**Ownership & borrowing.** `Wallet` owns its `UtxoSet` outright (a `HashMap`,
not references), so a wallet is a self-contained value that can be passed
around or dropped without lifetime parameters. `create_transaction` takes
`&mut self`: it reads the set to select inputs (`UtxoSet::select` borrows
`&self`), then mutates it in place to remove spent UTXOs and insert the
change output. Selection is fallible and checked *before* any mutation, so a
failed payment (e.g. `InsufficientFunds`) never leaves the wallet partially
spent — see `payment_beyond_balance_is_rejected_without_mutating_wallet` in
the integration tests.

**UTXO selection trade-offs.** `CoinSelectionStrategy` offers two strategies,
both implemented as a simple sort-then-greedy-accumulate over the candidate
UTXOs (no branch-and-bound / exact-match search — deliberately kept simple
for this assignment):

- `LargestFirst` — spend the biggest UTXOs first. Minimizes the number of
  inputs (smaller transaction, lower fee for a given payment), but breaks
  large UTXOs into a large-payment + change pair, and over many transactions
  tends to leave a wallet holding lots of small "dust" UTXOs that are
  expensive to spend later relative to their value.
- `SmallestFirst` — spend the smallest UTXOs first. Consolidates dust as a
  side effect of ordinary spending (fewer, larger UTXOs left over), at the
  cost of needing more inputs — and therefore a larger, costlier
  transaction — for the same payment.

Real wallets (e.g. Bitcoin Core's Branch and Bound / knapsack selection) try
to find a subset that matches the target close to exactly, to avoid creating
a change output at all. That's out of scope here; the two greedy strategies
were chosen because they make the size-vs-dust trade-off directly
observable and testable (see `largest_first_prefers_fewer_bigger_utxos` vs
`smallest_first_consolidates_dust` in `src/utxo.rs`).

**Transaction state extension (attempted).** `TransactionStatus` is an enum
(`Draft`, `Signed`, `Broadcast`, `Confirmed { height: u32 }`) tracked as
private state on `Transaction`. `advance_status` only allows the linear
progression `Draft -> Signed -> Broadcast -> Confirmed`; any other transition
(e.g. skipping straight to `Broadcast`, or moving backwards) returns
`TransactionError::InvalidStateTransition { from, to }`. This is exercised in
both `src/transaction.rs` and `main.rs`/`tests/payment_flow.rs`.

## Running

```
cargo test
cargo test -- --ignored   # no-op: nothing is ignored
cargo run
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Example output

```
$ cargo run
Mined:
Transaction coinbase-1 [draft]
  in:  coinbase (block 901450)
  out: 50000 sats -> bc1q-alice
Starting balance: 62900 sats

Built payment to Bob:
Transaction tx-1 [draft]
  in:  coinbase-1:0
  out: 20000 sats -> bc1q-bob
  out: 29750 sats -> bc1q-alice-change
Inputs: 50000 sats, fee: 250 sats
Status: confirmed at height 901452

Remaining balance: 42650 sats

Expected failure paying Carol: insufficient funds: required 1000250 sats, only 42650 sats available
```

## Part 7 Q&A

See [QUESTIONS.md](QUESTIONS.md) for short answers to the Part 7 conceptual
questions (inputs, outputs, UTXOs, fees, ownership/borrowing, `Result` vs
`panic!`, enums, traits), including a real `cargo build` ownership error and
what caused it.
