# Lab 07 — Confirmation and block membership

## Commands used

`cargo test --test lab_07
cargo run --example lab07_check
docker exec polar-n3-backend1 bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getblockcount
docker exec polar-n3-backend1 bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getblockhash 103
`

## Terminal output

`ConfirmationReport {
    txid: "a9d0febd729cf46b33a44e7a2007266ac1332b554cfd6f98aae864036701aaa9",
    block_hash: "2e69f34a27acdd327da77e12aaeb42e7812b0c5977392976b4e3b315a03698d9",
    confirmations: 3,
    mempool_is_empty: true,
    transaction_is_in_block: true,
}
`

## Evidence references

https://drive.google.com/drive/folders/1mP1ycuASg9SOfhFiHK00MdBMmprZZjQp?usp=drive_link

## Explanation

Mining did not change the transaction itself in any way. The TXID a9d0febd729cf46b33a44e7a2007266ac1332b554cfd6f98aae864036701aaa9 is identical across Lab 05 (when I broadcast it, unconfirmed), Lab 06 (when I decoded its exact inputs, outputs, and fee), and Lab 07 (this confirmation check) — same bytes, same signature, same value transfer. A transaction's identity is fixed the moment it's signed; nothing about mining rewrites or re-derives it.

What changed is purely positional. Before mining, the transaction existed in a kind of limbo: known to the node, broadcast to peers, sitting in the mempool — but not yet placed anywhere permanent, and in principle still droppable or replaceable. Mining a block that includes it converts that limbo into a fixed, provable position inside the agreed-upon chain history. My own evidence shows this concretely: transaction_is_in_block: true isn't a status the wallet just reports — I got it by fetching the actual block (getblock <hash> 1) and finding the TXID literally listed inside that block's tx array. That's a structural fact about the blockchain's data, not an opinion the wallet is offering.

The mempool side of the same coin shows up in mempool_is_empty: true. Once a transaction is included in a block, it has no reason to remain in the "waiting to be ordered" pool — it already has an order. Leaving the mempool and entering a block are the same event observed from two different places.

Finally, the confirmation count itself (3, not the 2 I initially expected) says nothing about the transaction changing — it reflects how much additional chain has been built on top of the block that already contains it. An extra block got mined outside my own code's control (most likely Polar's Auto Mine or a manual mine click in the UI) between when I confirmed the transaction and when I ran this check, and that alone was enough to bump the confirmation count from what I expected. The transaction was exactly as settled at 1 confirmation as it is at 3 — confirmations measure chain depth on top of a fixed point, not the transaction becoming "more true" over time.
