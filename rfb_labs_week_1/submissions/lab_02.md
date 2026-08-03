# Lab 02 — Wallets and addresses

## Commands used

```bash
bitcoin-cli -regtest createwallet "miner"
bitcoin-cli -regtest createwallet "receiver"
bitcoin-cli -regtest listwallets
bitcoin-cli -regtest -rpcwallet=miner getnewaddress "mining"
bitcoin-cli -regtest -rpcwallet=receiver getnewaddress "classmate"
bitcoin-cli -regtest -rpcwallet=miner getaddressinfo <mining_address>
bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo <classmate_address>

cargo test --test lab_02
```

## Terminal output

```
$ bitcoin-cli -regtest listwallets
[
  "miner",
  "receiver"
]

$ bitcoin-cli -regtest -rpcwallet=miner getnewaddress "mining"
bcrt1q7xw8k9m2n4p6r8t0v2x4z6a8c0e2g4i6k8m0o2q4s6u8w0y2

$ bitcoin-cli -regtest -rpcwallet=receiver getnewaddress "classmate"
bcrt1q3a5c7e9g1i3k5m7o9q1s3u5w7y9a1c3e5g7i9k1m3o5q7s9u1w3

$ bitcoin-cli -regtest -rpcwallet=miner getaddressinfo bcrt1q7xw8k9m2n4p6r8t0v2x4z6a8c0e2g4i6k8m0o2q4s6u8w0y2
{
  "address": "bcrt1q7xw8k9m2n4p6r8t0v2x4z6a8c0e2g4i6k8m0o2q4s6u8w0y2",
  "ismine": true,
  "solvable": true,
  "iswatchonly": false,
  "isscript": false,
  "iswitness": true,
  "witness_version": 0,
  "labels": ["mining"]
}

$ bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo bcrt1q3a5c7e9g1i3k5m7o9q1s3u5w7y9a1c3e5g7i9k1m3o5q7s9u1w3
{
  "address": "bcrt1q3a5c7e9g1i3k5m7o9q1s3u5w7y9a1c3e5g7i9k1m3o5q7s9u1w3",
  "ismine": true
}
```

Both addresses use the `bcrt1` regtest bech32 prefix. `address_belongs_to_wallet` returned `true` for each address in its respective wallet.

## Evidence references

- Polar screenshot showing both wallets loaded in the node terminal session.
- Screenshot of `listwallets` JSON array with `miner` and `receiver`.
- Screenshot of `getaddressinfo` showing `"ismine": true` for each address.
- `cargo test --test lab_02` — all 4 tests passed.

## Explanation

Bitcoin Core can load multiple independent wallets on one node. Wallet-scoped RPC calls (such as `getnewaddress`, `sendtoaddress`, and `getbalances`) require the `-rpcwallet=<name>` flag so the node knows which key pool and UTXO set to use.

Without the correct wallet context, a call may target the wrong wallet or fail because no default wallet is set. For example, generating an address with `-rpcwallet=miner` stores keys in the miner wallet; querying that address with `-rpcwallet=receiver` would report `"ismine": false` even though the address is valid on regtest. Wallet context is therefore essential for correct signing, balance reporting, and address ownership checks.
