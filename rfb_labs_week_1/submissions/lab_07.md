# Lab 07 — Confirmation and block membership

## Commands used

Ran `generatetoaddress 1`, `getrawmempool`, receiver `gettransaction`, `getbalances`, and `getblock <hash> 1`.

## Terminal output

Confirming block `20cd1e994aef0397db662221ac6d394d8c674ffe5ea997cb267b4426dd61f1bb` was height 102. Mempool was empty, receiver trusted balance was 1 BTC, the payment had one confirmation, and the block `tx` list contained `b0be49f7ba206557fe14510f6d6f00e16a9cf1d7155fb459babd97695a8b7fe0`.

## Evidence references

Captured `getblock` JSON reported `nTx: 2` and listed coinbase plus the payment; receiver JSON supplied the same block hash.

## Explanation

Mining did not change the signed transaction bytes. It placed the transaction in an accepted proof-of-work block, giving it an agreed position in chain history and one confirmation.
