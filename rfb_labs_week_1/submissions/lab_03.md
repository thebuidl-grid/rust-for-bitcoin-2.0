# Lab 03 — Coinbase maturity

## Commands used

```bash
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest generatetoaddress 1 bcrt1q60es4phrzmtqa5w6knzs8gqw97ykfn9kxfllwt
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest getblockcount
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=miner getbalances
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=miner sendtoaddress bcrt1qduu2p93zkvp0v69uqglj0dlaam0rwplhk6gf3u 1
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest generatetoaddress 100 bcrt1q60es4phrzmtqa5w6knzs8gqw97ykfn9kxfllwt
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest getblockcount
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=miner getbalances
```

## Terminal output

```text
First block: 08088785647283988da524169d3034dbbda83f633c299b97bf6b2c0d1bc99670
Height: 1
mine: { trusted: 0.00000000, untrusted_pending: 0.00000000, immature: 50.00000000 }

Premature send exit status: 6
error code: -6
error message:
Insufficient funds

100-block result: ["1369e781...d288f", ... 98 hashes omitted ..., "51857abd...e2f93"]
Height: 101
mine: { trusted: 50.00000000, untrusted_pending: 0.00000000, immature: 5000.00000000 }
```

## Evidence references

Live, sequential transcript from the dedicated Polar Regtest node. The failed payment was
captured before any additional maturity blocks were generated.

## Explanation

Bitcoin consensus applies `COINBASE_MATURITY = 100`: a coinbase output cannot be spent
until 100 blocks have been built after its block. At height 1 the first 50 BTC reward was
present but entirely `immature`, so the real 1 BTC send failed with Core error `-6`.
After mining 100 more blocks, the tip was height 101 and that first reward had 101
confirmations, making 50 BTC trusted/spendable. The 100 later rewards were newer and
remained immature, totaling 5,000 BTC. This is why a fresh-chain demonstration mines
1 + 100 = 101 blocks.
