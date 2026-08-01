# Lab 05 — Broadcast and mempool

## Commands used

```
cargo test --test lab_05
cargo fmt --check
BITCOIN_CLI=<bitcoin-cli wrapper> cargo run --example lab05_demo
```

Underlying RPCs (`src/labs/lab05_mempool.rs`), no block was mined between
sending and observing:
```
sendtoaddress bcrt1qxmst06m... 1   -rpcwallet=miner
getrawmempool
gettransaction <txid>              -rpcwallet=miner
getbalances                        -rpcwallet=receiver
```

## Terminal output

`cargo test --test lab_05`:
```
running 4 tests
test observes_broadcast_without_confirmation ... ok
test reads_local_mempool_txids ... ok
test reads_wallet_transaction_status ... ok
test sends_payment_in_sender_wallet_context ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`cargo run --example lab05_demo` against the live node (miner → receiver's
classmate address, exactly 1 BTC, no mining):
```
MempoolObservation {
    txid: "cfb0ea5976993f1245ada575b4472138ac9d91fcbea342068e82ef5ea29f1cfe",
    mempool_contains_tx: true,
    sender_status: WalletTransactionStatus {
        txid: "cfb0ea5976993f1245ada575b4472138ac9d91fcbea342068e82ef5ea29f1cfe",
        confirmations: 0,
        amount: -1.0,
        fee: Some(-2.82e-5),
        block_hash: None,
    },
    receiver_balance: WalletBalances {
        trusted: 0.0,
        untrusted_pending: 1.0,
        immature: 0.0,
    },
}
```

Cross-check directly on the node:
```
$ bitcoin-cli getrawmempool
["cfb0ea5976993f1245ada575b4472138ac9d91fcbea342068e82ef5ea29f1cfe"]

$ bitcoin-cli -rpcwallet=receiver getbalances
{ "mine": { "trusted": 0.0, "untrusted_pending": 1.0, "immature": 0.0 } }
```

## Evidence references

- Screenshot: `submissions/images/Screenshot from 2026-08-01 13-58-25.png` — IDE
  terminal running `cargo test --test lab_05`, all 4 tests passing.
- TXID `cfb0ea5976993f1245ada575b4472138ac9d91fcbea342068e82ef5ea29f1cfe`
  appears in `getrawmempool`'s output (`mempool_contains_tx: true`), proving
  the node relayed/accepted the broadcast transaction.
- The sender's own `gettransaction` view reports `confirmations: 0` and no
  `block_hash` — zero confirmations, matching a purely mempool-resident
  transaction.
- The receiver wallet's `getbalances` shows `untrusted_pending: 1.0` and
  `trusted: 0.0` — the incoming 1 BTC is visible but explicitly *not* counted
  as spendable/trusted funds yet.
- Together these three independent observations (mempool membership, 0
  confirmations, untrusted-pending balance) prove broadcast alone is not
  confirmation.

## Explanation

A Bitcoin transaction's lifecycle has four distinct states:

1. **Built and signed** — the transaction exists only as bytes on the sender's
   machine; nothing about it has been shared with the network yet.
2. **Broadcast** — the sender relays the signed transaction to peers via
   `sendtoaddress`/`sendrawtransaction`; other nodes may or may not have seen
   it yet, and it isn't part of anyone's confirmed history.
3. **Mempool** — nodes that received and validated the transaction hold it in
   their local mempool, a provisional pool of "next block" candidates. It has
   no permanent place in the blockchain and can still be evicted, replaced
   (RBF), or simply never mined if fees are too low.
4. **Confirmed** — a miner includes the transaction in a mined block; it now
   has a fixed position in the agreed-upon chain history and one confirmation,
   increasing with every block built on top.

This lab deliberately stops at state 3: the transaction is broadcast and
sitting in the mempool (`mempool_contains_tx: true`), the sender sees `0`
confirmations, and the receiver's balance is merely `untrusted_pending` — none
of that guarantees the payment will ultimately be confirmed, since until it's
mined the transaction could still be dropped or double-spent.
