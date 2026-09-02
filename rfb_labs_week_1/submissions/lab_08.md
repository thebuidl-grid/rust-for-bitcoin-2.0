# Lab 08 — Block security

## Commands used

```bash
cargo run -- lab08
```

```bash
bitcoin-cli ... getblockheader 437082f60d095e6233d0cef531f9b5ae193ec02aaf818d9126821dc46585fe32
bitcoin-cli ... -rpcwallet=receiver gettransaction f4ddd9cb43e8ec40c6ca4d34a2ba8407b285f5daefd93d1e73a3621ed82f532d
bitcoin-cli ... generatetoaddress 5 bcrt1qfsw0fvcdjruj7d746sxqy0nnnpptcvsyslhx0q
bitcoin-cli ... -rpcwallet=receiver gettransaction f4ddd9cb43e8ec40c6ca4d34a2ba8407b285f5daefd93d1e73a3621ed82f532d
```

The confirmation count is read once before mining and once after, so the change is
measured rather than assumed.

## Terminal output

```text
$ bitcoin-cli ... getblockheader 437082f6...fe32
  {
    "hash": "437082f60d095e6233d0cef531f9b5ae193ec02aaf818d9126821dc46585fe32",
    "confirmations": 1,
    "height": 103,
    "version": 536870912,
    "merkleroot": "798e557fb4814cce9f65a88174925e38f0687172ffcde76121641d0a75fd30f5",
    "time": 1785500868,
    "nonce": 0,
    "bits": "207fffff",
    "target": "7fffff0000000000000000000000000000000000000000000000000000000000",
    "difficulty": 4.656542373906925e-10,
    "chainwork": "00000000000000000000000000000000000000000000000000000000000000d0",
    "nTx": 2,
    "previousblockhash": "2bd214eac37965a28be72f17a1bdfddc1491477b56ec4f74fcbb2c6df792f511"
  }
```

Recorded header fields:

| Field | Value |
| --- | --- |
| Block hash | `437082f60d095e6233d0cef531f9b5ae193ec02aaf818d9126821dc46585fe32` |
| Height | 103 |
| Previous block | `2bd214eac37965a28be72f17a1bdfddc1491477b56ec4f74fcbb2c6df792f511` |
| Merkle root | `798e557fb4814cce9f65a88174925e38f0687172ffcde76121641d0a75fd30f5` |
| Nonce | 0 |
| Bits / target | `207fffff` / `7fffff00…0000` |
| Difficulty | 4.656542373906925e-10 |
| Confirmations | 1 |
| Chainwork | `00000000…000000d0` |

After mining five more blocks (ending at height 108):

```text
$ bitcoin-cli ... -rpcwallet=receiver gettransaction f4ddd9cb...532d
  {
    "amount": 1.00000000,
    "confirmations": 6,
    "blockhash": "437082f60d095e6233d0cef531f9b5ae193ec02aaf818d9126821dc46585fe32",
    "blockheight": 103
  }

confirmations 1 -> 6 after five blocks
```

The block hash and height did not move — only the depth beneath the tip changed.

## Evidence references

Full run log at `evidence/week1-labs-01-09.log`, lines 728-847, including the five block
hashes produced by `generatetoaddress 5` and both `gettransaction` calls.

## Explanation

**Hash links.** Every header carries `previousblockhash`, here
`2bd214ea…f511`. Since the header is hashed to produce the block's own identifier,
changing anything in block 102 changes its hash, which breaks the link stored in block
103, which changes 103's hash, and so on to the tip. The chain is not merely ordered — each
block cryptographically commits to its entire ancestry.

**Merkle commitment.** The header does not contain the transactions, only the Merkle root
`798e557f…30f5`, a single hash at the top of a binary tree over this block's two
transactions. It means a tiny fixed-size header commits to arbitrarily many transactions:
altering any one of them changes the root and therefore the header. It also allows proving
one transaction is in a block using a path of about log₂(n) hashes rather than the whole
block.

**Proof-of-work search.** `bits: 207fffff` encodes the target
`7fffff00…0000`. Miners repeatedly change the `nonce` (and other malleable header fields)
and re-hash the header until the result is numerically below that target. There is no
shortcut — it is brute force, and finding a valid header is evidence that real work was
performed. Regtest sets an almost trivial target, so `nonce: 0` succeeded on the very
first attempt. On mainnet the difficulty is roughly 10¹³ times higher.

**Chainwork and why confirmations matter.** `chainwork` `…00d0` is the *cumulative*
expected work across every block up to this one, not the work of block 103 alone. When my
payment had 1 confirmation, block 103 was the tip and a competitor needed only one block
to displace it. At 6 confirmations, five more blocks sit on top, and rewriting history to
exclude my payment means re-mining block 103 *and* out-pacing all five successors. The cost
grows with each block, which is why exchanges wait for six.

The crucial limit: confirmations make it expensive to *reverse* a valid transaction. They
do nothing to make an invalid one valid. Every node independently verifies signatures,
checks that inputs exist and are unspent, and confirms outputs never exceed inputs. A block
containing an invalid transaction is rejected outright no matter how much work backs it —
the work is simply discarded. Proof of work settles *which* valid history to agree on; it
never overrides the validity rules themselves.
