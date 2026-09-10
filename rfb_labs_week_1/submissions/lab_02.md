# Lab 02 — Wallets and addresses

## Commands used

cargo build
cargo run --bin lab_02_usage

## Terminal output

miner_wallet: miner
receiver_wallet: receiver
miner_address: bcrt1qwteesk497ndpw242fpvazn7w3ynkfdgvxcvre7
receiver_address: bcrt1qhk55m2g2hfd0xct7hsuvxp3j796fk2l6q7t62j
loaded_wallets: ["miner", "receiver"]
address_belongs_to_wallet_miner: true
address_belongs_to_wallet_receiver: true

## Evidence references

Two wallets are created, a new address is genereted in each wallet, and then is checked whether each generated address actually belongs to the expected wallet.

Loaded wallets are listed:
`loaded_wallets: ["miner", "receiver"]`


Address ownership is verified:
`address_belongs_to_wallet_miner: true`
`address_belongs_to_wallet_receiver: true`

 ## Explanation

“Wallet context” means: when someone calls an RPC method, it may be interpreted either globally (node-wide) or within a specific wallet (wallet state: keys, addresses, transactions, UTXOs, balance).