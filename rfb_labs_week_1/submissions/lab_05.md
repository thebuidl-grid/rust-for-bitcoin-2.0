# Lab 05 — Broadcast and mempool

## Commands used

Ran miner `sendtoaddress <receiver> 1`, `getrawmempool`, sender `gettransaction`, and receiver `getbalances`, without mining.

## Terminal output

TXID `b0be49f7ba206557fe14510f6d6f00e16a9cf1d7155fb459babd97695a8b7fe0` was in the mempool. Sender reported 0 confirmations and receiver showed trusted 0 BTC with untrusted-pending 1 BTC.

## Evidence references

Node A terminal JSON records the exact mempool TXID, sender fee `-0.00002820`, and receiver pending balance.

## Explanation

A signed transaction becomes broadcast when accepted and relayed, then sits in a node's mempool. It becomes confirmed only after a valid active-chain block contains it.
