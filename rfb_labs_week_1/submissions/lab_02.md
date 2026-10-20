# Lab 02 — Wallets and addresses

## Commands used

cargo run --example lab02

Underlying bitcoin-cli RPCs invoked:
- createwallet "miner"
- createwallet "receiver"
- listwallets
- getnewaddress "mining" (wallet: miner)
- getnewaddress "classmate" (wallet: receiver)
- getaddressinfo "bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn" (wallet: miner)
- getaddressinfo "bcrt1q6nlqtswveesh573tml9mrwlczn66vfu0sjnaqn" (wallet: receiver)

## Terminal output

loaded wallets: ["", "miner", "receiver"]
miner address (mining):       bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn
receiver address (classmate): bcrt1q6nlqtswveesh573tml9mrwlczn66vfu0sjnaqn
miner wallet owns mining address:       true
receiver wallet owns classmate address: true

## Evidence references

Polar's node Wallets tab shows miner and receiver as loaded wallets; each wallet's receive tab shows the generated address matching the terminal output above.

## Explanation

Wallet context is basically the use of wallet level RPCs to specifically let Bitcoin Core's RPC know which wallet you are referring to when you write your commands because Bitcoin Core supports loading multiple wallets at once. `-rpcwallet=<walletname>` is how you resolve that, it routes the call to the specific wallet's endpoint