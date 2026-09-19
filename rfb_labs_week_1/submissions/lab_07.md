# Lab 07 — Confirmation and block membership

## Commands used

TODO: Record the mining, mempool, transaction, and block commands.

Rust: `cargo run --example run` (calls `confirm_and_locate_transaction`, which runs):
- `generatetoaddress 1 <miner_address>` — mine one confirming block
- `getrawmempool` — verify the mempool emptied
- `gettransaction <txid>` (wallet: receiver) — read confirmation count and containing block hash
- `getblock <block_hash> 1` — verify the txid is listed in that block's transactions

## Terminal output


=== Lab 07: confirm transaction ===
ConfirmationReport {
txid: "83c922a38a18b50448317bdbcc334f51ff145910ef8e24793cf4a048613b48ec",
block_hash: "20cd23ecb5398b06d18f803acb034a128c2d706b22f059c03d8233c41f354e65",
confirmations: 1,
mempool_is_empty: true,
transaction_is_in_block: true,
}

## Evidence references

Screenshot: `evidence/lab07.png`

## Explanation

TODO: Explain exactly what changed when the transaction became confirmed.

Mining one block resolved the exact ambiguity from Lab 05. Before this, the transaction existed only as a broadcast, unconfirmed intent sitting in the mempool. After mining, four things changed simultaneously: the mempool emptied (`mempool_is_empty: true` — the transaction was pulled out and included, not left behind), `confirmations` moved from 0 to 1, `gettransaction` now reports an actual `block_hash` instead of `None`, and directly checking that block's transaction list with `getblock` confirms the txid is genuinely inside it (`transaction_is_in_block: true`) rather than just trusting the wallet's own claim.

Mining didn't alter the transaction itself in any way — the signed transaction data, its txid, and its inputs/outputs are byte-identical to what was broadcast in Lab 05. What changed is its *status*: it went from "a valid, signed transaction that exists" to "a transaction whose position in the one agreed-upon history of the blockchain is now fixed." Confirmation isn't about the transaction becoming more true — it's about the network reaching agreement on where it sits.
