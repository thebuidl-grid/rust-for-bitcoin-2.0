# Lab 08 — Block security

## Commands used

cargo run --example lab08 -- 16e63e070ff88f883247a9ee8208206b73348a8f401ab5b1d108d9884adea592

Underlying bitcoin-cli RPCs invoked:
- getblockheader 16e63e070ff88f883247a9ee8208206b73348a8f401ab5b1d108d9884adea592
- gettransaction a9e5849b95b19d9c08218953eeb0475c75b8b856f5838615bd37f37f6056647b   (wallet: receiver)
- generatetoaddress 5 bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn
- gettransaction a9e5849b95b19d9c08218953eeb0475c75b8b856f5838615bd37f37f6056647b   (wallet: receiver)


## Terminal output

header:
  hash:                16e63e070ff88f883247a9ee8208206b73348a8f401ab5b1d108d9884adea592
  height:              103
  previous_block_hash: Some("39192fb282dacb172ca1460f264fda7ed5070f7c14563f41f4d012fffe3457ec")
  merkle_root:         208aed596cb5422063384665413e6dbc0d81e12d4ac13539bc0a673a43d902f0
  nonce:               0
  difficulty:          0.00000000046565423739069247
  bits:                207fffff
  confirmations:       2
  chainwork:           00000000000000000000000000000000000000000000000000000000000000d0
confirmations_before:  2
confirmations_after:   7

## Evidence references

confirmations_before:  2
confirmations_after:   7

## Explanation

A hash link is what makes the chain a chain: every block header contains the hash of the previous block's header, so each block cryptographically references its predecessor, you can't alter a past block without changing its hash, which breaks the link every block built on top of it depends on, forcing you to redo all of those too.
A Merkle root is a single hash that summarizes every transaction in a block: transactions are hashed in pairs, those hashes are hashed in pairs again, and so on up to one root hash stored in the block header, this lets a node prove a specific transaction is included in a block using just a short "Merkle path" of hashes, without downloading every transaction in the block.
Proof of work is the mechanism that makes producing a valid block expensive: miners must find a header (which includes a nonce they can vary) whose hash falls below a target difficulty value, and since hashing is one-way and unpredictable, the only way to find one is brute-force trial and error, this is what makes rewriting history costly, since you'd have to redo that work for the block you're changing and every block after it, faster than the rest of the network is extending the real chain.
Confirmation depth is just the count of blocks built on top of the one containing your transaction, and it matters because of proof of work and hash links together: each additional block adds more accumulated work an attacker would need to outpace to reorg you out, so depth is a direct, quantifiable measure of how expensive it would be to reverse that transaction.