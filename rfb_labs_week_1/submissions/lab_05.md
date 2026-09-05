# Lab 05 — Broadcast and mempool

## Commands used

TODO: Record the payment, mempool, transaction, and balance commands.
Here's just that one section, ready to fill in:

---

**Commands used**

- `cargo test --test lab_05` — runs the public unit tests against a mocked RPC client.
- `bitcoin-cli -regtest -rpcwallet=receiver getnewaddress` — get an address to pay.
- `bitcoin-cli -regtest -rpcwallet=miner sendtoaddress <receiver-address> 1` — broadcast a 1 BTC payment without mining it.
- `bitcoin-cli -regtest getrawmempool` — confirm the transaction is sitting in the mempool.
- `bitcoin-cli -regtest -rpcwallet=miner gettransaction <txid>` — check the sender's view of the transaction (should show 0 confirmations).
- `bitcoin-cli -regtest -rpcwallet=receiver getbalances` — check the receiver's balance (payment should show as pending, not trusted).

## Terminal output

TODO: Show the TXID, zero confirmations, mempool entry, and pending balance.
labs_week_1$ cargo test --test lab_05
   Compiling rfb-labs-week-1 v0.1.0 (/home/jemiah/Documents/rustforbitcoin/rust-for-bitcoin-2.0/rfb_labs_week_1)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.23s
     Running tests/lab_05.rs (target/debug/deps/lab_05-f8379d3d71d8600a)

running 4 tests
test reads_local_mempool_txids ... ok
test observes_broadcast_without_confirmation ... ok
test reads_wallet_transaction_status ... ok
test sends_payment_in_sender_wallet_context ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

jemiah@jemiah-ThinkPad-X13-Gen-1:~/Documents/rustforbitcoin/rust-for-bitcoin-2.0/rfb_labs_week_1$ 

## Evidence references

TODO: Link screenshots or describe the attached evidence.
![alt text](image-4.png)

## Explanation

TODO: Distinguish signed, broadcast, mempool, and confirmed states.
Here's a simple version:

**Signed** — the sender's wallet has approved the transaction (proved with a digital signature) but hasn't sent it anywhere yet.

**Broadcast** — the signed transaction has been sent out to the Bitcoin network for other nodes to see.

**Mempool** — nodes that receive the transaction hold it in a waiting area (the "memory pool") until a miner picks it up and puts it in a block. It's visible to the network, but not yet locked in.

**Confirmed** — a miner included the transaction in a block, and that block is now part of the chain. This is the point where the payment is actually settled — before this, it could technically still be replaced or dropped.

Simple way to think about it: **signed** = "I approved this," **broadcast** = "I told everyone," **mempool** = "everyone's seen it but it's not final yet," **confirmed** = "it's locked into the blockchain now."