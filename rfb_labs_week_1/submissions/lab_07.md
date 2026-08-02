# Lab 07 — Confirmation and block membership

## Commands used

<!-- TODO: Record the mining, mempool, transaction, and block commands. -->
```bash
# Mine one block 
bitcoin-cli generatetoaddress 1 <addr>

# Get Transaction Details and check transaction confirmations
bitcoin-cli -rpcwallet=miner gettransaction <tx_hash>

# Get Transaction Details and verify transaction is in block
bitcoin-cli getblock  <block_hash>

# Verify mempool is empty
bitcoin-cli getrawmempool
```


## Terminal output

<!-- TODO: Show the empty mempool, confirmation count, block hash, and TXID in block. -->
```bash
bitcoin@backend1:/$ bitcoin-cli generatetoaddress 1 bcrt1qn893ldl3w0zt5myjm0lxh3kpreedtwtnsc0272
[
  "3099247c5d3142780f74c5ec4f80ec266b316ac297d03ec10bc8d55e4f129060"
]
bitcoin@backend1:/$ bitcoin-cli getrawmempool
[
]
bitcoin@backend1:/$ ^C
bitcoin@backend1:/$  bitcoin-cli -rpcwallet=miner gettransaction 5973f6d0548485a3abf7e0afb640a3a52e1
ed515b2eb94a32e9047174368d53f
{
  "amount": -10.00000000,
  "fee": -0.00002820,
  "confirmations": 2,
  "blockhash": "2435845577ede896b61bb1a9bada81c92762d152d3ae3e8f222f24d9689545f9",
  "blockheight": 103,
  "blockindex": 1,
  "blocktime": 1785587064,
  "txid": "5973f6d0548485a3abf7e0afb640a3a52e1ed515b2eb94a32e9047174368d53f",
  "wtxid": "e94420a677791be16709181715a3a53d1f9bb6e0536eb8f14de5bb576ae2aa81",
  "walletconflicts": [
  ],
  "mempoolconflicts": [
  ],
  "time": 1785582968,
  "timereceived": 1785582968,
  "bip125-replaceable": "no",
  "details": [
    {
      "address": "bcrt1qcs67lj6q6up4nadjggckcpek27wtrkgz8h58wr",
      "category": "send",
      "amount": -10.00000000,
      "vout": 0,
      "fee": -0.00002820,
      "abandoned": false
    }
  ],
  "hex": "020000000001018edbe67c082d50456a49b0c41243400abb626bcc9a54143628f9cbb409b6c7880000000000fdffffff0200ca9a3b00000000160014c435efcb40d70359f5b242316c0736579cb1d902fc1c6bee0000000016001410b93a0d8cac6adf7799f30b7b3c302ac2529d3102473044022051613181b3224c8078ed8a6beafe9205e927cd8f0fcde1166a0fed8d58b66b9f022018eb3d017b93203756f94c419ad1a6c0017cb53dc4eda03237cd33318e9f260e012102c1f38a06d513232a1163d22d0814828209ac7d0e61f569feddf87d5741c7f94030000000",
  "lastprocessedblock": {
    "hash": "3099247c5d3142780f74c5ec4f80ec266b316ac297d03ec10bc8d55e4f129060",
    "height": 104
  }
}
bitcoin@backend1:/$ bitcoin-cli gtblock 2435845577ede896b61bb1a9bada81c92762d152d3ae3e8f222f24d9689545f9 1                                                              
error code: -32601
error message:
Method not found
bitcoin@backend1:/$ bitcoin-cli getblock 2435845577ede896b61bb1a9bada81c92762d152d3ae3e8f222f24d9689
545f9 1
{
  "hash": "2435845577ede896b61bb1a9bada81c92762d152d3ae3e8f222f24d9689545f9",
  "confirmations": 2,
  "height": 103,
  "version": 536870912,
  "versionHex": "20000000",
  "merkleroot": "b6cb1ef452326fb849950b8a1d13041249b4f830fa5ed1ccc9e3b62f03089e30",
  "time": 1785587064,
  "mediantime": 1785582869,
  "nonce": 0,
  "bits": "207fffff",
  "target": "7fffff0000000000000000000000000000000000000000000000000000000000",
  "difficulty": 4.656542373906925e-10,
  "chainwork": "00000000000000000000000000000000000000000000000000000000000000d0",
  "nTx": 2,
  "previousblockhash": "30aee7e2f5b8671723de4af2eaf8776736522ad961aef220e10d79b0de3541a7",
  "nextblockhash": "3099247c5d3142780f74c5ec4f80ec266b316ac297d03ec10bc8d55e4f129060",
  "strippedsize": 326,
  "size": 471,
  "weight": 1449,
  "tx": [
    "3f5d18963cba928638b76f881d86a2ccfd616b87b6078c6c7ceb0ecec14fa5f9",
    "5973f6d0548485a3abf7e0afb640a3a52e1ed515b2eb94a32e9047174368d53f"
  ]
}
bitcoin@backend1:/$ 

```

## Evidence references

<!-- TODO: Link screenshots or describe the attached evidence. -->
screenshot shows the polar response for different methods called


screenshot shows the test passing for lab07 implementation.

## Explanation

<!-- TODO: Explain exactly what changed when the transaction became confirmed. -->

Before confirmation (mempool)

Transaction is valid and known to nodes, but not yet part of the blockchain.
Inputs (UTXOs being spent) are still technically "unspent" in the official UTXO set.
Outputs (new UTXOs being created) don't exist yet — nothing can spend them.

What happens at confirmation
A miner includes the tx in a mined block. That block gets added to the chain. This triggers:

Old UTXOs removed — the inputs the tx spent are deleted from the UTXO set (they're now spent, permanently).
New UTXOs created — the tx's outputs are added to the UTXO set as fresh, spendable coins.
Mempool entry removed — the tx is dropped from the mempool (it's no longer "pending," it's final).
Block height/chain state updates — the tx now has 1 confirmation, and its data (txid, block hash) is permanently recorded on-chain.
