# Lab 02 — Wallets and addresses

## Commands used

Ran `createwallet miner`, `createwallet receiver`, `listwallets`, wallet-scoped `getnewaddress`, and `getaddressinfo` with `-regtest`.

## Terminal output

Loaded wallets included `miner` and `receiver`. Their addresses were `bcrt1qh0hf6fkgrs9merk3xtc34dpfefgd02lyg4nnyy` and `bcrt1qtq62v48v6skk4awy5k6lnupc05qm9cq8w8qpz0`; both start `bcrt1` and returned `ismine: true` in their own wallet context.

## Evidence references

Node A terminal JSON includes the labels `mining` and `classmate`, both addresses, and the ownership checks.

## Explanation

`-rpcwallet` routes a wallet RPC to one named wallet. The wrong context can create data in, or query, the wrong wallet; `ismine` confirms actual wallet control.
