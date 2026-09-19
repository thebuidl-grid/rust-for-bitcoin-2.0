# Lab 05 — Broadcast and mempool

## Commands used

Rust: `cargo run --example run` (calls `observe_unconfirmed_payment`, which internally runs):
- `sendtoaddress <receiver_address> 1` (wallet: miner) — via `send_btc`
- `getrawmempool` — via `get_raw_mempool`
- `gettransaction <txid>` (wallet: miner) — via `get_transaction_status`
- `getbalances` (wallet: receiver) — via `get_balances`

## Terminal output

=== Lab 05: mempool ===
MempoolObservation {
txid: "83c922a38a18b50448317bdbcc334f51ff145910ef8e24793cf4a048613b48ec",
mempool_contains_tx: true,
sender_status: WalletTransactionStatus {
txid: "83c922a38a18b50448317bdbcc334f51ff145910ef8e24793cf4a048613b48ec",
confirmations: 0,
amount: -1.0,
fee: Some(
-2.82e-5,
),
block_hash: None,
},
receiver_balance: WalletBalances {
trusted: 0.0,
untrusted_pending: 1.0,
immature: 0.0,
},
}

## Evidence references

Screenshot: `evidence/lab05.png`

## Explanation

A Bitcoin transaction moves through distinct states before it's final, and this lab captures the middle one — broadcast but unconfirmed.

**Signed and broadcast:** `sendtoaddress` did three things internally — selected a UTXO to spend, built and signed the transaction, and broadcast it to the network. That returned a `txid` immediately, before any block had included it.

**In the mempool:** `getrawmempool` confirmed the transaction is sitting in the node's mempool (`mempool_contains_tx: true`) — the pool of valid, signed transactions that have been broadcast and are waiting to be picked up by a miner, but haven't been included in a block yet. It exists on the network at this point, but its place in the permanent transaction history isn't settled.

**Not yet confirmed:** `gettransaction` on the sender's side shows `confirmations: 0` and `block_hash: None` — direct proof no block has included it yet. From the sender's perspective, `amount: -1.0` reflects the wallet's own balance decreasing (a debit), with `fee: Some(-2.82e-5)` showing the tiny fee it paid to include the transaction.

**The receiver's view matters too:** the receiver wallet already sees the incoming 1 BTC, but specifically as `untrusted_pending` rather than `trusted` — the balance types aren't interchangeable. `untrusted_pending` money is visible and expected but not considered safe to spend or count on, precisely because an unconfirmed transaction can still theoretically be replaced or dropped before a miner ever includes it. Only after a block confirms it (Lab 07) does that balance move into `trusted`.
