# Lab 08 — Block security

## Commands used

TODO: Record block-header inspection and additional mining commands.

## Terminal output

TODO: Show header fields and confirmation count changing from one to six.

## Evidence references

TODO: Link screenshots or describe the attached evidence.

## Explanation

Hash links: each block header stores the hash of the previous block, so changing old data changes the chain’s hashes.
Merkle root: a cryptographic summary of all transactions in the block. It commits to which transactions are included.
Proof of Work: miners must find a block hash that meets the network difficulty target.
Confirmation depth: how many blocks have been added on top of the block containing the transaction.