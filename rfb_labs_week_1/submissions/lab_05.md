# Lab 05 — Broadcast and mempool

## Commands used

Rust:

```
cargo test --test lab_05
cargo fmt --check
cargo run --example lab05
```

`examples/lab05.rs` calls the completed `observe_unconfirmed_payment` function against the real
node — sending a fresh 1 BTC payment from `miner` to a new `receiver` address and inspecting it
without mining any blocks.

Bitcoin Core RPCs (run directly in Polar's node terminal, no mining in between):

```
bitcoin-cli -rpcwallet=miner sendtoaddress $RECEIVER_ADDR 1
bitcoin-cli getrawmempool
bitcoin-cli -rpcwallet=miner gettransaction <txid>
bitcoin-cli -rpcwallet=receiver getbalances
```

## Terminal output

`cargo test --test lab_05`:

```
running 4 tests
test reads_local_mempool_txids ... ok
test observes_broadcast_without_confirmation ... ok
test reads_wallet_transaction_status ... ok
test sends_payment_in_sender_wallet_context ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo run --example lab05` (real node, via the completed Rust implementation):

```
MempoolObservation {
    txid: "4b5fadc1ce7c5eb942bfe290c095e9904fa2f49564dcd472014c8cd57eb619c1",
    mempool_contains_tx: true,
    sender_status: WalletTransactionStatus {
        txid: "4b5fadc1ce7c5eb942bfe290c095e9904fa2f49564dcd472014c8cd57eb619c1",
        confirmations: 0,
        amount: -1.0,
        fee: Some(-2.82e-5),
        block_hash: None,
    },
    receiver_balance: WalletBalances {
        trusted: 0.0,
        untrusted_pending: 2.0,
        immature: 0.0,
    },
}
```
(`untrusted_pending` is `2.0` rather than `1.0` because this was the receiver's second unconfirmed
payment in this session, stacked on top of the manual one below — both are still legitimately
unconfirmed.)

Raw `bitcoin-cli` output from the manual walkthrough (cross-checking the same behavior on a
separate payment):

```
$ TXID=$(bitcoin-cli -rpcwallet=miner sendtoaddress $RECEIVER_ADDR 1)
$ echo $TXID
f29961f07a5a57137b43cd46d05f89df2b685eb605296ffe03519955b87da3ef

$ bitcoin-cli getrawmempool
[ "f29961f07a5a57137b43cd46d05f89df2b685eb605296ffe03519955b87da3ef" ]

$ bitcoin-cli -rpcwallet=miner gettransaction $TXID
{
  "amount": -1.0,
  "fee": -0.0000282,
  "confirmations": 0,
  "txid": "f29961f07a5a57137b43cd46d05f89df2b685eb605296ffe03519955b87da3ef",
  ...
}

$ bitcoin-cli -rpcwallet=receiver getbalances
{
  "mine": { "trusted": 0.0, "untrusted_pending": 1.0, "immature": 0.0 }
}
```

Both the Rust implementation and the raw RPC calls agree: the TXID is present in
`getrawmempool`, the sender's own wallet reports `confirmations: 0` and no `block_hash` for it,
and the receiver's wallet shows the incoming amount as `untrusted_pending`, not `trusted` — none of
it usable yet, but visibly on its way.

## Evidence references

Terminal output above was captured directly from Polar's node terminal and from
`cargo run --example lab05`; no separate screenshots were taken for this lab.

## Explanation

A Bitcoin payment passes through several distinct states, and "sent" doesn't mean "final":

1. **Built and signed** — the wallet selects UTXOs to spend, constructs the transaction, and signs
   it with the relevant private keys. At this point it exists only locally; no one else has seen it.
2. **Broadcast** — the transaction is announced to the node's peers. `sendtoaddress` does both the
   building/signing and the broadcasting in one RPC call.
3. **Mempool (unconfirmed)** — once broadcast, nodes that accept it as valid hold it in their local
   mempool, a holding area of transactions waiting to be included in a block. This is exactly what
   `getrawmempool` reports. A transaction can sit here for a while (or, in principle, never get
   mined) — being broadcast is not a guarantee of eventual confirmation.
4. **Confirmed** — only once a miner includes the transaction in a block that gets added to the
   chain does it become confirmed, at which point `gettransaction` would start reporting a real
   `confirmations` count above 0 and a `block_hash`.

The receiver's `untrusted_pending` balance reflects step 3, not step 4: Bitcoin Core can see the
incoming payment sitting in the mempool and knows it's addressed to one of the wallet's own
addresses, but won't count it as trusted, spendable money until it's actually confirmed — because
an unconfirmed transaction can still be dropped, replaced (this transaction was even marked
`"bip125-replaceable": "yes"`), or simply never mined at all.
