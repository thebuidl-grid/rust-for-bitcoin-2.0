# Lab 08 — Block security

## Commands used

<!-- TODO: Record block-header inspection and additional mining commands. -->
```bash
# Inspect block header
bitcoin-cli -getblockheader <block-hash>

# Check transaction confirmations
bitcoin-cli -rpcwallet=<walletr> gettransaction <tx-hash>

# Mine additional blocks
bitcoin-cli generatetoadress 5 <address>

```

## Terminal output

<!-- TODO: Show header fields and confirmation count changing from one to six. -->
```bash
bitcoin@backend1:/$ 
bitcoin@backend1:/$ bitcoin-cli getblockheader 3099247c5d3142780f74c5ec4f80ec266b316ac297d03ec10bc8d55e4f129060
{
  "hash": "3099247c5d3142780f74c5ec4f80ec266b316ac297d03ec10bc8d55e4f129060",
  "confirmations": 1,
  "height": 104,
  "version": 536870912,
  "versionHex": "20000000",
  "merkleroot": "6623e46db6c2b2e41cc8bf117ad51f6d2f9be1cfebf99a68cc36d31f4e035968",
  "time": 1785645427,
  "mediantime": 1785582869,
  "nonce": 0,
  "bits": "207fffff",
  "target": "7fffff0000000000000000000000000000000000000000000000000000000000",
  "difficulty": 4.656542373906925e-10,
  "chainwork": "00000000000000000000000000000000000000000000000000000000000000d2",
  "nTx": 2,
  "previousblockhash": "2435845577ede896b61bb1a9bada81c92762d152d3ae3e8f222f24d9689545f9"
}
bitcoin@backend1:/$ bitcoin-cli generatetoaddress 5 bcrt1qn893ldl3w0zt5myjm0lxh3kpreedtwtnsc0272
[
  "208fec97f6632a0cfb3af0989e79632bda87b990e09c59e8506ac6d93a91b030",
  "34634c236a1695294cac13902623dc5a5d9fcff557aaa9733e942f10e2de9713",
  "5a874464fcab2889008212526647a1cd7e198b67b235bb5bdf74bc0bab0cdadb",
  "3d93005fe116c485cbfc2ad277b2331d54aee7bbe239bcc372f9bbd3cf38dacc",
  "39c9ffd1357409f1bb145b2a1746b95fd4599bf044619f50e447dd1261aeaa14"
]
bitcoin@backend1:/$ bitcoin-cli getblockheader 3099247c5d3142780f74c5ec4f80ec266b316ac297d03ec10bc8d55e4f129060
{
  "hash": "3099247c5d3142780f74c5ec4f80ec266b316ac297d03ec10bc8d55e4f129060",
  "confirmations": 6,
  "height": 104,
  "version": 536870912,
  "versionHex": "20000000",
  "merkleroot": "6623e46db6c2b2e41cc8bf117ad51f6d2f9be1cfebf99a68cc36d31f4e035968",
  "time": 1785645427,
  "mediantime": 1785582869,
  "nonce": 0,
  "bits": "207fffff",
  "target": "7fffff0000000000000000000000000000000000000000000000000000000000",
  "difficulty": 4.656542373906925e-10,
  "chainwork": "00000000000000000000000000000000000000000000000000000000000000d2",
  "nTx": 2,
  "previousblockhash": "2435845577ede896b61bb1a9bada81c92762d152d3ae3e8f222f24d9689545f9",
  "nextblockhash": "208fec97f6632a0cfb3af0989e79632bda87b990e09c59e8506ac6d93a91b030"
}
bitcoin@backend1:/$ 
```

## Evidence references

<!-- TODO: Link screenshots or describe the attached evidence. -->
Screenshot of bitcoin-cli methods on polar terminal, showing the block confirmation, mining additional 5 blocks , then showing the block confirmation after.


Sreenshot of lab09 implementation passing 


## Explanation

<!-- TODO: Explain hash links, Merkle roots, proof of work, and confirmation depth. -->

Hash links
Each block contains the hash of the previous block's header. This chains blocks together — you can't change an old block without breaking every hash pointing to it in blocks after it. This is what makes it a "blockchain."

Merkle root
A single hash that summarizes all transactions in a block. Transactions are hashed in pairs, repeatedly, up to one final hash (the root). Change any transaction → the Merkle root changes → the block header changes → the block's hash changes. Lets you verify a transaction is in a block without downloading every transaction.

Proof of Work (PoW)
Miners must find a block header hash below a target value — done by brute-force guessing a nonce. This takes real computational effort, making it costly to produce a valid block and even costlier to redo one (since redoing one means redoing every block after it too).

Confirmation depth
How many blocks have been mined on top of the block containing your transaction. Each additional block = more accumulated proof-of-work protecting it. Depth 1 = just mined. Depth 6 = generally considered final/irreversible for most purposes, since reversing it would require outpacing the entire network's honest mining power.

How they connect: Hash links make the chain tamper-evident. Merkle roots let each block efficiently prove its transaction set. PoW makes rewriting history expensive. Confirmation depth is how you measure "how expensive it would be to undo this now."