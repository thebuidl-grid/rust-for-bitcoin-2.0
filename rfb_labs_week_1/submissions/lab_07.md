# Lab 07 — Confirmation and block membership

## Commands used

TODO: Record the mining, mempool, transaction, and block commands.

# ==========================================
# 1. SETUP: Get a destination address
# ==========================================
DEST=$(docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 getnewaddress)
echo "Destination Address: $DEST"

# ==========================================
# 2. PAYMENT: Send a transaction
# ==========================================
TXID=$(docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 -named sendtoaddress address="$DEST" amount=0.5)
echo "Transaction ID (TXID): $TXID"

# ==========================================
# 3. MEMPOOL: Inspect before mining
# ==========================================
echo -e "\n--- Mempool Contents (TXIDs) ---"
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getrawmempool

echo -e "\n--- Mempool Entry Details ---"
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getmempoolentry "$TXID"

# ==========================================
# 4. MINING: Include transaction in a block
# ==========================================
MINER_ADDR=$(docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 getnewaddress)
echo -e "\n--- Mining 1 Block ---"
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass generatetoaddress 1 "$MINER_ADDR"

# ==========================================
# 5. BLOCK: Inspect the new block
# ==========================================
BLOCK_HASH=$(docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getbestblockhash)
echo -e "\n--- Block Details (Verbose) ---"
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getblock "$BLOCK_HASH" 2

# ==========================================
# 6. TRANSACTION: Verify confirmation & Decode
# ==========================================
echo -e "\n--- Transaction Status (Confirmations) ---"
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass gettransaction "$TXID"

echo -e "\n--- Verbose Transaction Decode (Level 2) ---"
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getrawtransaction "$TXID" 2

# ==========================================
# 7. BALANCE: Final Wallet State
# ==========================================
echo -e "\n--- Final Wallet Balances ---"
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 getbalances   

## Terminal output

TODO: Show the empty mempool, confirmation count, block hash, and TXID in block.

--- Mempool Contents (TXIDs) ---
[
  "9536e1ed08a56b89e45a69d2ade2a84b1068d62a92997b5b6e084d7ef486042e"
]

--- Mempool Entry Details ---
{
  "vsize": 141,
  "weight": 561,
  "time": 1785758434,
  "height": 405,
  "descendantcount": 1,
  "descendantsize": 141,
  "ancestorcount": 1,
  "ancestorsize": 141,
  "wtxid": "e00ee1d660d82d192c2ddaf441937ac1416fcfc100aee733fc2cdc0636be22cd",
  "fees": {
    "base": 0.00002820,
    "modified": 0.00002820,
    "ancestor": 0.00002820,
    "descendant": 0.00002820
  },
  "depends": [
  ],
  "spentby": [
  ],
  "bip125-replaceable": true,
  "unbroadcast": true
}

--- Mining 1 Block ---
[
  "2116cb4ddde5b503d418595cfdb7dc03725b5776a68e4bf625fa00cf362e89c1"
]

--- Block Details (Verbose) ---
{
  "hash": "2116cb4ddde5b503d418595cfdb7dc03725b5776a68e4bf625fa00cf362e89c1",
  "confirmations": 1,
  "height": 406,
  "version": 805306368,
  "versionHex": "30000000",
  "merkleroot": "0be5d0ffece420e4b91938c6bfd3b8ff8049ad428b736b1eed69496b33614955",
  "time": 1785758434,
  "mediantime": 1785757102,
  "nonce": 5,
  "bits": "207fffff",
  "difficulty": 4.656542373906925e-10,
  "chainwork": "000000000000000000000000000000000000000000000000000000000000032e",
  "nTx": 2,
  "previousblockhash": "5a80a437d646773c9f6f04fa70f21d09b996a99743687e6814a29d3fb3fa8084",
  "strippedsize": 327,
  "size": 472,
  "weight": 1453,
  "tx": [
    {
      "txid": "273e18ab89efeaf04a301222c0130f2e3c778eb953e7e2aaa227c475ff46983f",
      "hash": "c677524a65e57d23a0adfaf30bb8fc0b37158e09b01a1c8e52097bf23f2b511b",
      "version": 2,
      "size": 169,
      "vsize": 142,
      "weight": 568,
      "locktime": 0,
      "vin": [
        {
          "coinbase": "02960100",
          "txinwitness": [
            "0000000000000000000000000000000000000000000000000000000000000000"
          ],
          "sequence": 4294967295
        }
      ],
      "vout": [
        {
          "value": 12.50002820,
          "n": 0,
          "scriptPubKey": {
            "asm": "0 f4e3ae9dfeb1d3275bf3435e25090ce1c75c3d8c",
            "desc": "addr(bcrt1q7n36a807k8fjwklngd0z2zgvu8r4c0vvttaxph)#5xn87zzk",
            "hex": "0014f4e3ae9dfeb1d3275bf3435e25090ce1c75c3d8c",
            "address": "bcrt1q7n36a807k8fjwklngd0z2zgvu8r4c0vvttaxph",
            "type": "witness_v0_keyhash"
          }
        },
        {
          "value": 0.00000000,
          "n": 1,
          "scriptPubKey": {
            "asm": "OP_RETURN aa21a9edb29136cf2fbd9e9c3ca7d09e5c7b056df4ec8c7159200296f565c9ea45650239",
            "desc": "raw(6a24aa21a9edb29136cf2fbd9e9c3ca7d09e5c7b056df4ec8c7159200296f565c9ea45650239)#9f3wqgu3",
            "hex": "6a24aa21a9edb29136cf2fbd9e9c3ca7d09e5c7b056df4ec8c7159200296f565c9ea45650239",
            "type": "nulldata"
          }
        }
      ],
      "hex": "020000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff0402960100ffffffff028487814a00000000160014f4e3ae9dfeb1d3275bf3435e25090ce1c75c3d8c0000000000000000266a24aa21a9edb29136cf2fbd9e9c3ca7d09e5c7b056df4ec8c7159200296f565c9ea456502390120000000000000000000000000000000000000000000000000000000000000000000000000"
    },
    {
      "txid": "9536e1ed08a56b89e45a69d2ade2a84b1068d62a92997b5b6e084d7ef486042e",
      "hash": "e00ee1d660d82d192c2ddaf441937ac1416fcfc100aee733fc2cdc0636be22cd",
      "version": 2,
      "size": 222,
      "vsize": 141,
      "weight": 561,
      "locktime": 405,
      "vin": [
        {
          "txid": "4768d3534b3160661813fd0f6f1d83cd1f9f0dac0a44a6d6f4a0616f2ca5f65e",
          "vout": 0,
          "scriptSig": {
            "asm": "",
            "hex": ""
          },
          "txinwitness": [
            "304402200b8108a415e97ce95a95a8edc4c41994eb9b27db12c824e499aa54217252a6aa02207a20817639e402c8b86a74fb70fe3164773deb3b7f1eaafae7f0dab26023f12b01",
            "02021030071775faed9cc201873ec397af9dd83a10b140f160b6a8947c15f608a0"
          ],
          "sequence": 4294967293
        }
      ],
      "vout": [
        {
          "value": 0.49993020,
          "n": 0,
          "scriptPubKey": {
            "asm": "0 256784bcb0877e16dd2cd6add8faaee160796096",
            "desc": "addr(bcrt1qy4ncf09ssalpdhfv66ka374wu9s8jcykzty38u)#kemevx78",
            "hex": "0014256784bcb0877e16dd2cd6add8faaee160796096",
            "address": "bcrt1qy4ncf09ssalpdhfv66ka374wu9s8jcykzty38u",
            "type": "witness_v0_keyhash"
          }
        },
        {
          "value": 0.50000000,
          "n": 1,
          "scriptPubKey": {
            "asm": "0 3030999b4bb78bfdf567677856241c156af1eca2",
            "desc": "addr(bcrt1qxqcfnx6tk79lmat8vau9vfquz440rm9ztdx9v7)#xeqsjrk8",
            "hex": "00143030999b4bb78bfdf567677856241c156af1eca2",
            "address": "bcrt1qxqcfnx6tk79lmat8vau9vfquz440rm9ztdx9v7",
            "type": "witness_v0_keyhash"
          }
        }
      ],
      "fee": 0.00002820,
      "hex": "020000000001015ef6a52c6f61a0f4d6a6440aac0d9f1fcd831d6f0ffd13186660314b53d368470000000000fdffffff023cd5fa0200000000160014256784bcb0877e16dd2cd6add8faaee16079609680f0fa02000000001600143030999b4bb78bfdf567677856241c156af1eca20247304402200b8108a415e97ce95a95a8edc4c41994eb9b27db12c824e499aa54217252a6aa02207a20817639e402c8b86a74fb70fe3164773deb3b7f1eaafae7f0dab26023f12b012102021030071775faed9cc201873ec397af9dd83a10b140f160b6a8947c15f608a095010000"
    }
  ]
}

--- Transaction Status (Confirmations) ---
error code: -19
error message:
Wallet file not specified (must request wallet RPC through /wallet/<filename> uri-path).
Try adding "-rpcwallet=<filename>" option to bitcoin-cli command line.

--- Verbose Transaction Decode (Level 2) ---
{
  "txid": "9536e1ed08a56b89e45a69d2ade2a84b1068d62a92997b5b6e084d7ef486042e",
  "hash": "e00ee1d660d82d192c2ddaf441937ac1416fcfc100aee733fc2cdc0636be22cd",
  "version": 2,
  "size": 222,
  "vsize": 141,
  "weight": 561,
  "locktime": 405,
  "vin": [
    {
      "txid": "4768d3534b3160661813fd0f6f1d83cd1f9f0dac0a44a6d6f4a0616f2ca5f65e",
      "vout": 0,
      "scriptSig": {
        "asm": "",
        "hex": ""
      },
      "txinwitness": [
        "304402200b8108a415e97ce95a95a8edc4c41994eb9b27db12c824e499aa54217252a6aa02207a20817639e402c8b86a74fb70fe3164773deb3b7f1eaafae7f0dab26023f12b01",
        "02021030071775faed9cc201873ec397af9dd83a10b140f160b6a8947c15f608a0"
      ],
      "prevout": {
        "generated": false,
        "height": 405,
        "value": 0.99995840,
        "scriptPubKey": {
          "asm": "0 99611584e9ebc69c32573ae8485803539627a311",
          "desc": "addr(bcrt1qn9s3tp8fa0rfcvjh8t5yskqr2wtz0gc3z9376w)#ds2dh5dn",
          "hex": "001499611584e9ebc69c32573ae8485803539627a311",
          "address": "bcrt1qn9s3tp8fa0rfcvjh8t5yskqr2wtz0gc3z9376w",
          "type": "witness_v0_keyhash"
        }
      },
      "sequence": 4294967293
    }
  ],
  "vout": [
    {
      "value": 0.49993020,
      "n": 0,
      "scriptPubKey": {
        "asm": "0 256784bcb0877e16dd2cd6add8faaee160796096",
        "desc": "addr(bcrt1qy4ncf09ssalpdhfv66ka374wu9s8jcykzty38u)#kemevx78",
        "hex": "0014256784bcb0877e16dd2cd6add8faaee160796096",
        "address": "bcrt1qy4ncf09ssalpdhfv66ka374wu9s8jcykzty38u",
        "type": "witness_v0_keyhash"
      }
    },
    {
      "value": 0.50000000,
      "n": 1,
      "scriptPubKey": {
        "asm": "0 3030999b4bb78bfdf567677856241c156af1eca2",
        "desc": "addr(bcrt1qxqcfnx6tk79lmat8vau9vfquz440rm9ztdx9v7)#xeqsjrk8",
        "hex": "00143030999b4bb78bfdf567677856241c156af1eca2",
        "address": "bcrt1qxqcfnx6tk79lmat8vau9vfquz440rm9ztdx9v7",
        "type": "witness_v0_keyhash"
      }
    }
  ],
  "fee": 0.00002820,
  "hex": "020000000001015ef6a52c6f61a0f4d6a6440aac0d9f1fcd831d6f0ffd13186660314b53d368470000000000fdffffff023cd5fa0200000000160014256784bcb0877e16dd2cd6add8faaee16079609680f0fa02000000001600143030999b4bb78bfdf567677856241c156af1eca20247304402200b8108a415e97ce95a95a8edc4c41994eb9b27db12c824e499aa54217252a6aa02207a20817639e402c8b86a74fb70fe3164773deb3b7f1eaafae7f0dab26023f12b012102021030071775faed9cc201873ec397af9dd83a10b140f160b6a8947c15f608a095010000",
  "blockhash": "2116cb4ddde5b503d418595cfdb7dc03725b5776a68e4bf625fa00cf362e89c1",
  "confirmations": 1,
  "time": 1785758434,
  "blocktime": 1785758434
}

--- Final Wallet Balances ---
{
  "mine": {
    "trusted": 11287.49987380,
    "untrusted_pending": 0.00000000,
    "immature": 1250.00012620
  },
  "lastprocessedblock": {
    "hash": "2116cb4ddde5b503d418595cfdb7dc03725b5776a68e4bf625fa00cf362e89c1",
    "height": 406
  }
}

## Evidence references

TODO: Link screenshots or describe the attached evidence.

https://drive.google.com/drive/folders/1HvmkTC2bazkXgBELjgbLaaW8grJQgF9h?usp=sharing


## Explanation

TODO: Explain exactly what changed when the transaction became confirmed.

Mining did not change the transaction itself in any way. The TXID a9d0febd729cf46b33a44e7a2007266ac1332b554cfd6f98aae864036701aaa9 is identical across Lab 05 (when I broadcast it, unconfirmed), Lab 06 (when I decoded its exact inputs, outputs, and fee), and Lab 07 (this confirmation check) — same bytes, same signature, same value transfer. A transaction's identity is fixed the moment it's signed; nothing about mining rewrites or re-derives it.

What changed is purely positional. Before mining, the transaction existed in a kind of limbo: known to the node, broadcast to peers, sitting in the mempool — but not yet placed anywhere permanent, and in principle still droppable or replaceable. Mining a block that includes it converts that limbo into a fixed, provable position inside the agreed-upon chain history. My own evidence shows this concretely: transaction_is_in_block: true isn't a status the wallet just reports — I got it by fetching the actual block (getblock <hash> 1) and finding the TXID literally listed inside that block's tx array. That's a structural fact about the blockchain's data, not an opinion the wallet is offering.

The mempool side of the same coin shows up in mempool_is_empty: true. Once a transaction is included in a block, it has no reason to remain in the "waiting to be ordered" pool — it already has an order. Leaving the mempool and entering a block are the same event observed from two different places.

Finally, the confirmation count itself (3, not the 2 I initially expected) says nothing about the transaction changing — it reflects how much additional chain has been built on top of the block that already contains it. An extra block got mined outside my own code's control (most likely Polar's Auto Mine or a manual mine click in the UI) between when I confirmed the transaction and when I ran this check, and that alone was enough to bump the confirmation count from what I expected. The transaction was exactly as settled at 1 confirmation as it is at 3 — confirmations measure chain depth on top of a fixed point, not the transaction becoming "more true" over time.