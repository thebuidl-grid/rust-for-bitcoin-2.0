# Lab 08 — Block security

## Commands used

TODO: Record block-header inspection and additional mining commands.

# --- 1. Block Header Inspection ---
echo "=== BLOCK HEADER INSPECTION ==="
HASH=$(docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getbestblockhash)
echo "Current Best Block Hash: $HASH"
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getblockheader "$HASH" true

# --- 2. Additional Mining ---
echo -e "\n=== MINING 10 BLOCKS ==="
ADDR=$(docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 getnewaddress)
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass generatetoaddress 10 "$ADDR"

# --- 3. Verification ---
echo -e "\n=== CHAIN STATUS ==="
echo "New Block Height:"
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getblockcount
echo "New Best Block Hash:"
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getbestblockhash   

## Terminal output

TODO: Show header fields and confirmation count changing from one to six.

=== BLOCK HEADER INSPECTION ===
Current Best Block Hash: 2116cb4ddde5b503d418595cfdb7dc03725b5776a68e4bf625fa00cf362e89c1
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
  "previousblockhash": "5a80a437d646773c9f6f04fa70f21d09b996a99743687e6814a29d3fb3fa8084"
}

=== MINING 10 BLOCKS ===
[
  "7691ad105b8c0631b6d94a6a97796a694e66e6f8bff7a3c5fab7f8363fe1fb80",
  "2240c914dd83ab895c0a38593c4998b4b1d785643ca86854c329d5389c80adce",
  "6f5eed1216e317e3f187da8a5f31627700e1bec67eb5b338e422caa933918e65",
  "18e51ce242b37bae9169b3aa8597d6767a86da05243f71216ed0581228b1c887",
  "7c0aa040355cd59c82376559d1499f80a19191f46044a5b0ee92d1000d9ea457",
  "05a569131d51973acd586ff3302db8568ec5638fd5821c756e9b29e2dcbf954c",
  "12a02a2e7ca076439c49e7a466a9f162ecaa606110c7ecbf18660b8a2c94c6f2",
  "398f0605eed95c81f725b085eb39fb38a550af4a1f59d681997bd4369f95e9f2",
  "68ae93a02e5e4cc910acaac7e2d03460c1012d121fb2ff2cdb4eeb67b7ed1d87",
  "58c1d29702b0e0327c98f9930a5be9432b4f17c4589b6a6b11bca4f97e877faf"
]

=== CHAIN STATUS ===
New Block Height:
416
New Best Block Hash:
58c1d29702b0e0327c98f9930a5be9432b4f17c4589b6a6b11bca4f97e877faf

## Evidence references

TODO: Link screenshots or describe the attached evidence.

https://drive.google.com/drive/folders/1HvmkTC2bazkXgBELjgbLaaW8grJQgF9h?usp=sharing


## Explanation

TODO: Explain hash links, Merkle roots, proof of work, and confirmation depth.

A block header ties three separate mechanisms together to make the blockchain tamper-evident. The hash link is previous_block_hash — my block at height 103 explicitly embeds the hash of the block before it (28a3aa19...). This means every header is cryptographically bound to its predecessor: if anyone altered a past block, that block's own hash would change, which would break the previous_block_hash reference stored in the next block, cascading forward through every block mined since. Rewriting history isn't just "editing a record" — it requires re-mining the altered block and every single block built on top of it.

The Merkle commitment is the merkle_root field — a single hash that summarizes every transaction inside the block, Changing even one byte of one transaction in that block would produce a completely different Merkle root, which would change the block's own hash, which would in turn break the hash link described above. This is what lets a block header "commit" to an entire set of transactions using a fixed-size, 32-byte value, without needing to store the full transaction list in the header itself.

The proof-of-work search is represented by nonce and bits. bits (207fffff here) encodes the target difficulty a valid block hash must meet; nonce is the value miners increment while repeatedly hashing the header, searching for a result that satisfies that target. On my regtest chain, bits is set to the easiest possible target (difficulty: 4.66e-10), which is why blocks mine instantly

Validity and confirmation depth are answering two completely different questions. A transaction's validity — correct signatures, no double-spend, outputs not exceeding inputs — is checked against a fixed set of consensus rules the instant it's considered, and no amount of additional mining changes that answer. Piling blocks on top of an already-valid transaction cannot retroactively make a broken transaction correct.

What confirmations actually buy is economic finality against reorganization. To reverse a transaction buried N blocks deep, an attacker would need to build an alternative chain that is longer (has more accumulated work) than the honest chain from that point forward — meaning they'd have to out-mine N blocks' worth of proof-of-work from scratch while the rest of the network keeps extending the honest chain in parallel. My own report shows this accumulating directly: chainwork is a running total of proof-of-work invested in the entire chain up to that block, and it only grows. Each additional confirmation isn't "more proof the transaction is correct" — it's more accumulated work an attacker would have to overcome to erase it, which is why six confirmations is treated as far safer than one, even though both are equally "valid" from a rules standpoint.
