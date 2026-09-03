# Lab 03 — Coinbase maturity

## Commands used

Ran `generatetoaddress 1`, `getbalances`, attempted `sendtoaddress <receiver> 1`, then ran `generatetoaddress 100` and `getblockcount`.

## Terminal output

At height 1, miner had trusted `0.00000000` and immature `50.00000000` BTC; the payment failed with `Insufficient funds`. At height 101, trusted was `50.00000000` and immature was `5000.00000000` BTC.

## Evidence references

The Node A terminal records height-1 block `796cb68b9c87a6c0d718fc3fc2eb56e34ccd38c1c4f89c495019577060a9765d` and final height 101.

## Explanation

Coinbase rewards require 100 confirmations. The first reward is immature at height 1 but has sufficient confirmations at height 101; later rewards remain immature to limit reorganization risk.
