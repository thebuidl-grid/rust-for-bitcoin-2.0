# Lab 02 — Wallets and receiving addresses

## Commands used
- `bitcoin-cli -regtest createwallet "miner"` / `"receiver"` / `"alice"` backs `create_wallet`
- `bitcoin-cli -regtest listwallets` backs `list_wallets`
- `bitcoin-cli -regtest -rpcwallet=receiver getnewaddress "classmate"` backs `get_new_address`
- `bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo <address>` backs `address_belongs_to_wallet`

## Terminal output
$ bitcoin-cli -regtest listwallets
[
"miner",
"receiver",
"alice"
]

$ bitcoin-cli -regtest -rpcwallet=receiver getnewaddress "classmate"
bcrt1q82tmmnmf77qymd5kg6k7ly3k4a45pfp8l9xxlr

$ bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo bcrt1q82tmmnmf77qymd5kg6k7ly3k4a45pfp8l9xxlr
{
"address": "bcrt1q82tmmnmf77qymd5kg6k7ly3k4a45pfp8l9xxlr",
"scriptPubKey": "00143a97bdcf69f7804db69646adef9236af6b40a427",
"ismine": true,
"solvable": true,
"iswatchonly": false,
"isscript": false,
"iswitness": true,
"witness_version": 0,
"witness_program": "3a97bdcf69f7804db69646adef9236af6b40a427",
"ischange": false,
"hdkeypath": "m/84'/1'/0'/0/0",
"hdmasterfingerprint": "4fc4c7a4",
"labels": [
"classmate"
]
}
## Evidence references

Captured directly from a local Bitcoin Core node running in regtest mode
(bare-metal `~/.bitcoin`, three wallets loaded: miner, receiver, alice).
Address `bcrt1q82tmmnmf77qymd5kg6k7ly3k4a45pfp8l9xxlr` was generated in the
`receiver` wallet with label "classmate", and `getaddressinfo` confirms
`ismine: true` for that same wallet proving wallet-address ownership.

## Explanation (co-authored by Claude)

Bitcoin Core supports multiple independent wallets loaded simultaneously on one node, each with its own keys, addresses, and balances, useful for simulating separate parties (miner, receiver, alice) on a single node instance. Because several wallets can be loaded at once, most wallet-specific RPCs need to know which wallet to operate on; the -rpcwallet=<name> flag (or wallet-path routing) tells bitcoin-cli which wallet's context to use for that call. Without it, Bitcoin Core either defaults to a single loaded wallet or errors if more than one is loaded. getnewaddress derives a fresh receiving address from that wallet's HD keychain, and getaddressinfo lets you check whether a given address's private key is controlled by that wallet (ismine), which is how ownership is verified.

