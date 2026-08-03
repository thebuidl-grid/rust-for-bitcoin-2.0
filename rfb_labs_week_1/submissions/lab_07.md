# Lab 07 — Confirmation and block membership

## Commands used

Rust:

```
cargo test --test lab_07
cargo fmt --check
cargo run --example lab07
```

`examples/lab07.rs` calls the completed `confirm_and_locate_transaction` function against the real
node — it sends a fresh 1 BTC payment, then mines and confirms it end-to-end.

Bitcoin Core RPCs (run directly in Polar's node terminal, cross-checking the Lab 05/06 payment
after it was mined):

```
bitcoin-cli getrawmempool
bitcoin-cli -rpcwallet=miner gettransaction <txid>
bitcoin-cli -rpcwallet=receiver getbalances
bitcoin-cli getblock <blockhash> 1
```

## Terminal output

`cargo test --test lab_07`:

```
running 4 tests
test detects_empty_mempool ... ok
test mines_exactly_one_block ... ok
test reads_confirmation_count ... ok
test proves_transaction_is_inside_confirming_block ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo run --example lab07` (real node, full send-mine-confirm cycle, via the completed Rust
implementation):

```
sent txid: 8ca3af42693c8e673adfe6646eb199773251ffac82246e6b71d49c0e06696dda
ConfirmationReport {
    txid: "8ca3af42693c8e673adfe6646eb199773251ffac82246e6b71d49c0e06696dda",
    block_hash: "535b5cc57779211880def055af0290c92ecf969de98f9c7891c5063c773f723e",
    confirmations: 1,
    mempool_is_empty: true,
    transaction_is_in_block: true,
}
```

Raw `bitcoin-cli` output cross-checking the earlier Lab 05/06 payment
(`f29961f07a5a57137b43cd46d05f89df2b685eb605296ffe03519955b87da3ef`) after it was mined:

```
$ bitcoin-cli getrawmempool
[]

$ bitcoin-cli -rpcwallet=miner gettransaction f29961f0...
{
  "confirmations": 1,
  "blockhash": "568c4f7b0e70e388368ea69f9549cfb1093875f3044c0808d250ab8de7f237ba",
  "blockheight": 204,
  "txid": "f29961f07a5a57137b43cd46d05f89df2b685eb605296ffe03519955b87da3ef",
  ...
}

$ bitcoin-cli -rpcwallet=receiver getbalances
{ "mine": { "trusted": 3.0, "untrusted_pending": 0.0, "immature": 0.0 } }

$ bitcoin-cli getblock 568c4f7b0e70e388368ea69f9549cfb1093875f3044c0808d250ab8de7f237ba 1
{
  "hash": "568c4f7b0e70e388368ea69f9549cfb1093875f3044c0808d250ab8de7f237ba",
  "confirmations": 1,
  "height": 204,
  "nTx": 4,
  "tx": [
    "6d5019a7bdc3b1208dce0f134ad6dc02855525ae2d5b2f402e1997b3a1d9f490",
    "54ad0250cf0628ce9532d0d5e319761cac364036aec08aedcd305cbc2ab4bf55",
    "4b5fadc1ce7c5eb942bfe290c095e9904fa2f49564dcd472014c8cd57eb619c1",
    "f29961f07a5a57137b43cd46d05f89df2b685eb605296ffe03519955b87da3ef"
  ]
}
```

Every check lines up: the mempool no longer contains the TXID, the wallet reports 1 confirmation
and a concrete `blockhash`, the receiver's balance moved from `untrusted_pending` to fully
`trusted`, and the confirming block's own `tx` array lists the TXID as its 4th (final) entry.

## Evidence references

Terminal output above was captured directly from Polar's node terminal and from
`cargo run --example lab07`; no separate screenshots were taken for this lab.

## Explanation

Mining doesn't change anything about the transaction itself — its bytes, its inputs, its outputs,
its TXID are all identical before and after. What changes is the transaction's **place in the
agreed-upon history**: instead of sitting in the mempool as one node's (or the network's)
opinion of a plausible pending transaction, it's now bundled into a block that every node
following the same chain has accepted as part of the permanent, ordered ledger. That's the entire
meaning of "confirmed" — not that the transaction was somehow altered or made "more valid," but
that it now has a fixed, agreed position in a specific block, referenced by that block's hash, at a
specific height, alongside every other transaction that block contains.

This is also why `mempool_is_empty` and `transaction_is_in_block` are two separate proofs rather
than one: leaving the mempool only shows the transaction *stopped being pending locally* — it says
nothing on its own about whether it actually made it into a block (it could, in principle, have
been evicted or replaced). Confirming it's inside the block's own `tx` list is the real proof that
this exact transaction is now part of the chain everyone agrees on.
