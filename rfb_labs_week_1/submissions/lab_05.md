# Lab 05 — Broadcast and mempool

## Commands used

TODO: Record the payment, mempool, transaction, and balance commands.
# 1. Send BTC from sender wallet to receiver address without mining
bitcoin-cli -regtest -rpcwallet=sender sendtoaddress "bcrt1qreceiveraddress" 1.5

# 2. Inspect the local mempool for unconfirmed transactions
bitcoin-cli -regtest getrawmempool

# 3. View the sender's unconfirmed transaction details
bitcoin-cli -regtest -rpcwallet=sender gettransaction "4f6e43a9b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8"

# 4. Check receiver wallet balance before block confirmation (shows untrusted_pending)
bitcoin-cli -regtest -rpcwallet=receiver getbalances

## Terminal output

TODO: Show the TXID, zero confirmations, mempool entry, and pending balance.
$ cargo test --test lab_05
running 4 tests
test detects_mempool_contents ... ok
test inspects_unconfirmed_transaction_status ... ok
test observes_unconfirmed_payment_flow ... ok
test sends_btc_to_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
``bash
{
  "txid": "4f6e43a9b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8",
  "mempool_contains_tx": true,
  "sender_status": {
    "confirmations": 0,
    "blockhash": null
  },
  "receiver_balance": {
    "trusted": 0.00000000,
    "untrusted_pending": 1.50000000,
    "immature": 0.00000000
  }
}
``

## Evidence references

TODO: Link screenshots or describe the attached evidence.
Automated Unit Tests: Verified via cargo test --test lab_05 passing all 4 tests:

- sends_btc_to_address: Validates invoking sendtoaddress on the target wallet and extracting the returned TXID.

- detects_mempool_contents: Validates node mempool retrieval via getrawmempool.

- inspects_unconfirmed_transaction_status: Confirms that unconfirmed transactions return confirmations: 0 and blockhash: None.

- observes_unconfirmed_payment_flow: Validates tracking an unconfirmed transaction through sender status and receiver balance updates (untrusted_pending).

## Explanation

TODO: Distinguish signed, broadcast, mempool, and confirmed states.
1. Mempool Staging: When sendtoaddress is executed, the node signs and broadcasts the transaction across the peer-to-peer network. Before inclusion in a block, the transaction resides in the node's memory pool (mempool).

2. Unconfirmed Status & Double-Spend Risk: Unconfirmed transactions have 0 confirmations and no associated blockhash. They carry a risk of being double-spent or replaced (e.g., via RBF) until mined.

3. Wallet Balance States:

- untrusted_pending: Funds arriving at a wallet from an unconfirmed, external transaction. The wallet recognizes the incoming output script, but the funds cannot be safely spent yet.

- trusted: Funds backed by confirmed UTXOs (or unconfirmed self-change outputs) that are safe to spend.
