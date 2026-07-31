# Lab 01 — Regtest network inspection

## Commands used

```bash
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest invalidateblock 496498b148c5cadba69758d33b7b1d839ef15c0bfe05e7f594d3ddd0142876e5
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest getblockchaininfo
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest getblockcount
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest getbestblockhash
```

Polar started this new network with one initialization block. I invalidated only that
block before creating wallets so the maturity exercise could begin from exact fresh-chain
height 0.

## Terminal output

```text
{
  "chain": "regtest",
  "blocks": 0,
  "headers": 0,
  "bestblockhash": "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
  "bits": "207fffff",
  "difficulty": 4.656542373906925e-10,
  "initialblockdownload": false,
  "chainwork": "0000000000000000000000000000000000000000000000000000000000000002",
  "pruned": false
}

getblockcount: 0
getbestblockhash: 0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206
```

## Evidence references

Live Polar/Docker transcript above. Polar showed **Week 1 Bitcoin Fundamentals** in
`Started` state with one Bitcoin Core v30.0 node named `backend1`; Docker reported the
container `polar-n2-backend1` as running.

## Explanation

Polar set up the network and Docker ran the Bitcoin Core container. Bitcoin Core handled
validation and exposed the RPC methods used in the lab. Its response showed `regtest` at
height 0 with the regtest genesis block as the current tip. Regtest lets me generate local
blocks on demand, and its coins have no real-world value.
