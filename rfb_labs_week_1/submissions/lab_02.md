# Lab 02 — Wallets and addresses

## Commands used

cargo test --test lab_02
cargo run --example lab02_check

## Terminal output

`loaded wallets: ["", "miner", "receiver"]
mining address: bcrt1qq6jewkpw6yv97xpxkt8yf2j33p68fhe7kn4sfc
classmate address: bcrt1q0uwx0lqm5p8geyd0njz0hjmcl5fnrdl6ka56at
mining address belongs to miner: true
classmate address belongs to receiver: true`


## Evidence references
https://drive.google.com/drive/folders/1mP1ycuASg9SOfhFiHK00MdBMmprZZjQp?usp=drive_link

## Explanation

A single running Bitcoin Core node can have multiple wallets loaded simultaneously.But many RPC methods are inherently wallet-scoped — they need to know which wallet's keys/UTXOs/history to operate on as the method name alone doesn't specify that so rpcwallet<name> is how we supply that missing piece.

node-level calls such as createwallet and list wallets needed no wallet contexts so i passed `None`  but wallet-scoped calls such as getnewaddress, getaddressinfo needed `Some(wallet_name)`
