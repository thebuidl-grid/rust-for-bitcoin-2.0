# Lab 05 — Broadcast and mempool

## Commands used

cargo build
cargo run --bin lab_05_usage

## Terminal output

miner_wallet: miner
receiver_wallet: receiver
miner_address: bcrt1qehe9zgxq6xj8yv9c9xn74tvrmdrm6wm9z9suq8
101 blocks mined
receiver_address: bcrt1qzktea80cwjhj80grxl8n9hferfdt5uh9tjap34
payment txid: 
"0f44c9a16ed6552cad50a3ae6196567ad4d46ddcd6220b3da5484e2e9a8bb2af"
mempool: Ok(["0f44c9a16ed6552cad50a3ae6196567ad4d46ddcd6220b3da5484e2e9a8bb2af"])
transaction_status: WalletTransactionStatus { txid: "0f44c9a16ed6552cad50a3ae6196567ad4d46ddcd6220b3da5484e2e9a8bb2af", confirmations: 0, amount: 1.0, fee: None, block_hash: None }
Balance: 
WalletBalances {
    trusted: 14.0,
    untrusted_pending: 2.0,
    immature: 0.0,
}


## Evidence references

Creates two wallets on the Bitcoin test node: miner and receiver.
Generates a new mining address for the miner wallet and mines 101 blocks to mature/confirm funds.
Generates a new receiving address for the receiver wallet.
Sends 1.0 BTC from the miner wallet to the receiver address, capturing the resulting txid.
Fetches and prints the current mempool contents.
Retrieves and prints the transaction status for the receiver wallet and that txid.
Observes an unconfirmed payment for the sender/receiver context and prints the observed receiver balances.


## Explanation

Signed: The transaction has been created and cryptographically signed by the required inputs private keys, so it’s valid to broadcast.
Broadcast: The signed transaction has been sent to the network for propagation.
Mempool: The transaction is sitting in the node’s memory pool—currently unconfirmed.
Confirmed: The transaction has been included in a block and has at least one confirmation.