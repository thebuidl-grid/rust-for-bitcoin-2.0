# Lab 07 — Confirmation and block membership

## Commands used

TODO: Record the mining, mempool, transaction, and block commands.
```bash
bitcoin@backend1:/$ bitcoin-cli generatetoaddress 1 "$MINER_ADDR"
bitcoin@backend1:/$ bitcoin-cli -rpcwallet=miner1 getrawmempool
```

## Terminal output

TODO: Show the empty mempool, confirmation count, block hash, and TXID in block.
```bash
bitcoin@backend1:/$ bitcoin-cli generatetoaddress 1 "$MINER_ADDR"  
[
  "0870b121141d1575257a2a73c0875a521e9dbfca15e20e586c8ea3ccd2f3bf61"
]

bitcoin@backend1:/$ bitcoin-cli -rpcwallet=miner1 getrawmempool
[
  "31320512472671286073dcff374a9a93fdedfef7b1a11ce5ce4b01a020c4d38b"
]

bitcoin@backend1:/$ bitcoin-cli -rpcwallet=receiver1 gettransaction "31320512472671286073dcff374a9a93fdedfef7b1a11ce5ce4b01a020c4d38b"
{
  "amount": 1.00000000,
  "confirmations": 0,
  "trusted": false,
  "txid": "31320512472671286073dcff374a9a93fdedfef7b1a11ce5ce4b01a020c4d38b",
  "wtxid": "06b7f02d3bd52a243f400e1d1819cd2311e387538880e307a86f3f5b9e724650",
  "walletconflicts": [
  ],
  "mempoolconflicts": [
  ],
  "time": 1785500316,
  "timereceived": 1785500316,
  "bip125-replaceable": "yes",
  "details": [
    {
      "address": "bcrt1q9pl7mxzk0rqe5r2hh4vpp8vx5cxpaavejnh4d8",
      "parent_descs": [
        "wpkh([332e1503/84h/1h/0h]tpubDDXzSj9hzRtkwEpNJ4ck4eqXNhPgS9JyfYNXt79UmhYPvVuDjV8skv9rP8QJCKKs9nmj9S8X6cEQ28gjkQ1p6xtESMUAEBJ5Dkzan662tXS/0/*)#s9a5cnq7"
      ],
      "category": "receive",
      "amount": 1.00000000,
      "label": "receiving1",
      "vout": 1,
      "abandoned": false
    }
  ],
  "hex": "020000000001014c05206fb16b1b9e508eedf68262c160d74607366c514338da0162be2da42a790000000000fdffffff0204230d8f00000000160014a1cd8ba61f605f2ff45c128df10fbc27b32aa3d000e1f50500000000160014287fed985678c19a0d57bd58109d86a60c1ef599024730440220516698331be12bb7d5acfe837c3c3978bdeb11827f65e0c9ae60e2ee9bb6b911022013efc71db5b2c485c4f58b902f0a453bdb747efeaffc77b70e7e758d3932405e012103dadc5e2cd24a3c9722ab1c3e8c8ab95a6eb8a6b68dc7920f46bc2dba5b5693b236010000",
  "lastprocessedblock": {
    "hash": "5c3513c33fb0ea5144901aca1a8106615f303d4893fae3d373de5f3ee4d88187",
    "height": 310
  }
}
```

## Evidence references

TODO: Link screenshots or describe the attached evidence.
![img_8.png](evidence/img_8.png)

## Explanation

TODO: Explain exactly what changed when the transaction became confirmed.
The transaction moves from the mempool into a validated block, which updates the ledger by officially consuming old outputs and creating new, spendable UTXOs
Its confirmation count increases from zero, burying the transaction under proof of work
