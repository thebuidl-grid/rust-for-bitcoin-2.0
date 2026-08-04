# Lab 02 — Wallets and addresses

## Commands used

```bash
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest createwallet miner
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest createwallet receiver
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest listwallets
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=miner getnewaddress mining
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=receiver getnewaddress classmate
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=miner getaddressinfo bcrt1q60es4phrzmtqa5w6knzs8gqw97ykfn9kxfllwt
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo bcrt1qduu2p93zkvp0v69uqglj0dlaam0rwplhk6gf3u
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=miner getaddressinfo bcrt1qduu2p93zkvp0v69uqglj0dlaam0rwplhk6gf3u
```

## Terminal output

```text
createwallet miner:    { "name": "miner" }
createwallet receiver: { "name": "receiver" }
listwallets: ["", "miner", "receiver"]

miner/mining:       bcrt1q60es4phrzmtqa5w6knzs8gqw97ykfn9kxfllwt
receiver/classmate: bcrt1qduu2p93zkvp0v69uqglj0dlaam0rwplhk6gf3u

miner context, mining address:       { "ismine": true,  "iswatchonly": false, "solvable": true }
receiver context, classmate address: { "ismine": true,  "iswatchonly": false, "solvable": true }
miner context, classmate address:    { "ismine": false, "iswatchonly": false, "solvable": false }
```

## Evidence references

Live transcript from the `backend1` terminal in the Polar network named **Week 1 Bitcoin
Fundamentals**. The output is reduced to ownership fields so no wallet backup or private
material is included.

## Explanation

`listwallets` showed that `miner` and `receiver` were loaded, along with Polar's default
empty-name wallet. I used `-rpcwallet` because address and balance calls need the wallet
whose keys are being checked. The receiver address returned `ismine: true` in `receiver`
and `false` in `miner`, which confirmed the wallet boundary. Both addresses start with
`bcrt1`, the Bech32 prefix used on regtest.
