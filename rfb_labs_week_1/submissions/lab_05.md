# Lab 05 — Broadcast and mempool

## Commands used

cargo test --test lab_05
cargo run --example lab05_check

## Terminal output

`MempoolObservation {
    txid: "a9d0febd729cf46b33a44e7a2007266ac1332b554cfd6f98aae864036701aaa9",
    mempool_contains_tx: true,
    sender_status: WalletTransactionStatus {
        txid: "a9d0febd729cf46b33a44e7a2007266ac1332b554cfd6f98aae864036701aaa9",
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
`

## Evidence references

https://drive.google.com/drive/folders/1mP1ycuASg9SOfhFiHK00MdBMmprZZjQp?usp=drive_link


## Explanation

`The four transaction states: built and signed, broadcast, mempool, confirmed

A Bitcoin transaction doesn't jump straight from "intention" to "done" — it passes through distinct, observable states, and my Lab 05 run let me capture the middle two directly.

Built and signed happens entirely offline, before anything touches the network. When I called sendtoaddress, the miner wallet selected an existing UTXO (the 50 BTC matured coinbase reward from Lab 04), constructed a new transaction paying 1 BTC to the receiver's address plus a change output back to itself, and signed it with its own private key. Nothing outside the wallet process is involved yet — the transaction is fully valid and complete, but no other node or peer knows it exists.

Broadcast is the moment the wallet hands that signed transaction to the node's networking layer, which relays it to connected peers. In my case, the call to sendtoaddress returned a TXID (a9d0febd...) the instant this happened — that returned TXID is proof the node accepted and propagated the transaction, not proof it's final.

Mempool is where the transaction lives after broadcast but before a block includes it. My observe_unconfirmed_payment call proved this directly: mempool_contains_tx: true showed the TXID sitting in the node's local mempool, and sender_status.confirmations: 0 with block_hash: None confirmed no block has claimed it yet. Meanwhile, the receiver's wallet could already see the incoming payment — receiver_balance.untrusted_pending: 1.0 — but it wasn't trusted or spendable, only trusted: 0.0, because mempool membership isn't the same as settlement.
`
