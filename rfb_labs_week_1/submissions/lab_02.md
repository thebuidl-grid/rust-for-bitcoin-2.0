# Lab 02 — Wallets and addresses

## Commands used

`cargo test --test lab_02`

Live Polar node RPC commands:

`docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie createwallet miner`

`docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie createwallet receiver`

`docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie listwallets`

`docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner getnewaddress mining`

`docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=receiver getnewaddress classmate`

`docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner getaddressinfo bcrt1qz5qlp5nndqqnejnssgsqj9gz7c4garr40relsa`

`docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=receiver getaddressinfo bcrt1qkjlykz05rzkce4nc7lefska0zgy7j6vd9j7xw0`

## Terminal output

The public test suite passed, and the live node returned:

- loaded wallets: `""`, `miner`, `receiver`
- miner address: `bcrt1qz5qlp5nndqqnejnssgsqj9gz7c4garr40relsa`
- receiver address: `bcrt1qkjlykz05rzkce4nc7lefska0zgy7j6vd9j7xw0`
- miner ownership: `ismine: true`
- receiver ownership: `ismine: true`

Both addresses use the `bcrt1...` regtest prefix.

## Evidence references

Screenshot of the Polar Bitcoin Core terminal or Docker terminal showing the wallet creation, wallet list, address generation, and `getaddressinfo` checks on `polar-n1-backend1`.

## Explanation

`-rpcwallet` tells Bitcoin Core which loaded wallet should handle a wallet-scoped RPC call. That matters because `getnewaddress` and `getaddressinfo` belong to a specific wallet, not the node globally. If I use the wrong wallet context, Bitcoin Core may generate an address for the wrong wallet or report that the address is not mine. In this lab, the miner wallet must generate the mining address, and the receiver wallet must generate and verify the classmate address.
