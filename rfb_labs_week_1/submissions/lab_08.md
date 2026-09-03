# Lab 08 — Block security

## Commands used

Ran `getblockheader 20cd1e994aef0397db662221ac6d394d8c674ffe5ea997cb267b4426dd61f1bb`, mined five more blocks, and queried receiver `gettransaction`.

## Terminal output

The height-102 header has parent `11d69b4ad880d6279847a8fbdf5c9de3232003250c28f17833ba20a00d985592`, Merkle root `7a7d4235fb300eb7fe82b9ca37bd78d9149165c6f7ab114b824ebd74d83a7f40`, nonce 0, bits `207fffff`, and chainwork ending `00ce`. The payment advanced from 1 to exactly 6 confirmations.

## Evidence references

Node A `getblockheader` JSON records height, hashes, root, nonce, bits, target, difficulty, confirmations, and chainwork.

## Explanation

Parent hashes link blocks and the Merkle root commits to the transaction set. Proof of work makes replacement costly. Each additional confirmation increases reorganization cost, but never turns an invalid transaction into a valid one.
