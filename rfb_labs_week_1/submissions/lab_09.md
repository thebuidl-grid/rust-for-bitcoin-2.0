# Lab 09 — Multi-UTXO coin selection

## Commands used

```bash
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest createwallet alice
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=alice getnewaddress funding
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=miner sendtoaddress bcrt1qdjyr57e5jxtpu46c7dutfjrmdtxazq8yndmze4 0.4  # repeated 3 times
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest generatetoaddress 1 bcrt1q60es4phrzmtqa5w6knzs8gqw97ykfn9kxfllwt
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=alice listunspent
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=alice sendtoaddress bcrt1qw5ust93qhsddpp6rpu7z8y6yuhtf0lp2kfkec3 1
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest getrawtransaction d01712e4f2752b09a2d3fbcd6f665f8c8bb437cf1b4b02a89015f2e060c376ee 2
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest gettxout da65eb25e86d737d7d57936e84cbac5f7c473d14ef71fdf7d86be6d50e120aae 0 false
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest gettxout a0a6241475e240c807ddb68a5b3a76186c8f81ded67247158cc8743f7ae922a9 1 false
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest gettxout 37d0e73abf1879f55557a9a616548b12733df7ece3885d823f01f340107b934f 0 false
```

## Terminal output

```text
Alice address: bcrt1qdjyr57e5jxtpu46c7dutfjrmdtxazq8yndmze4
Confirmed UTXOs (all spendable, 1 confirmation, 0.40000000 BTC each):
  da65eb25e86d737d7d57936e84cbac5f7c473d14ef71fdf7d86be6d50e120aae:0
  a0a6241475e240c807ddb68a5b3a76186c8f81ded67247158cc8743f7ae922a9:1
  37d0e73abf1879f55557a9a616548b12733df7ece3885d823f01f340107b934f:0

Combined spend: d01712e4f2752b09a2d3fbcd6f665f8c8bb437cf1b4b02a89015f2e060c376ee
vsize: 276
vin: all three outpoints above
vout 0: 1.00000000 BTC -> bcrt1qw5ust93qhsddpp6rpu7z8y6yuhtf0lp2kfkec3
vout 1: 0.19994480 BTC -> bcrt1qrgw5j4e5fxrklna263m6q68rauzls5p57pwark

input sum: 120,000,000 sats
payment:   100,000,000 sats
change:     19,994,480 sats
fee:             5,520 sats
balanced: true
```

## Evidence references

Live Alice wallet, `listunspent`, verbose transaction, and previous-output transcripts.
Each listed funding TXID came from a separate `sendtoaddress` result before one common
confirmation block was mined.

## Explanation

Each input spends a complete UTXO, so Alice needed all three 0.4 BTC outputs to cover a
1 BTC payment and the fee. The receiver got exactly 1 BTC, 0.19994480 BTC returned to a
new Alice change address, and the remaining 5,520 sats paid the fee. Combining the three
inputs also links them on-chain. Someone analyzing the transaction may guess that one
owner controlled all three inputs, although that heuristic is not always correct.
