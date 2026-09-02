# Lab 02 — Wallets and addresses

## Commands used

```bash
cargo run -- lab02
```

Which runs, against node A (base arguments trimmed to `...` below):

```bash
bitcoin-cli ... createwallet miner
bitcoin-cli ... createwallet receiver
bitcoin-cli ... listwallets
bitcoin-cli ... -rpcwallet=miner    getnewaddress mining
bitcoin-cli ... -rpcwallet=receiver getnewaddress classmate
```

To prove ownership I deliberately asked each wallet about **both** addresses, so the
negative answers are part of the evidence:

```bash
bitcoin-cli ... -rpcwallet=miner    getaddressinfo bcrt1qfsw0fvcdjruj7d746sxqy0nnnpptcvsyslhx0q
bitcoin-cli ... -rpcwallet=receiver getaddressinfo bcrt1qfsw0fvcdjruj7d746sxqy0nnnpptcvsyslhx0q
bitcoin-cli ... -rpcwallet=receiver getaddressinfo bcrt1qga5wdzs456gvrk7kzh07lxxm5lxjarslkxm3m4
bitcoin-cli ... -rpcwallet=miner    getaddressinfo bcrt1qga5wdzs456gvrk7kzh07lxxm5lxjarslkxm3m4
```

## Terminal output

```text
$ bitcoin-cli ... listwallets
  [
    "",
    "miner",
    "receiver"
  ]

$ bitcoin-cli ... -rpcwallet=miner getnewaddress mining
  bcrt1qfsw0fvcdjruj7d746sxqy0nnnpptcvsyslhx0q
$ bitcoin-cli ... -rpcwallet=receiver getnewaddress classmate
  bcrt1qga5wdzs456gvrk7kzh07lxxm5lxjarslkxm3m4
```

Ownership, checked from both sides:

```text
miner    owns bcrt1qfsw0fvcdjruj7d746sxqy0nnnpptcvsyslhx0q: true
receiver owns bcrt1qfsw0fvcdjruj7d746sxqy0nnnpptcvsyslhx0q: false
receiver owns bcrt1qga5wdzs456gvrk7kzh07lxxm5lxjarslkxm3m4: true
miner    owns bcrt1qga5wdzs456gvrk7kzh07lxxm5lxjarslkxm3m4: false
```

The full `getaddressinfo` for the mining address shows `"ismine": true`, `"labels":
["mining"]`, and `"hdmasterfingerprint": "aa7f8695"`. Querying the same address through
`receiver` returns `"ismine": false` and `"solvable": false`, and the descriptor and
fingerprint fields are absent entirely — that wallet has no key for it.

Both addresses start with `bcrt1`, the regtest bech32 prefix, and both are
`witness_v0_keyhash` (P2WPKH).

## Evidence references

Full run log at `evidence/week1-labs-01-09.log`, lines 41-158, covering wallet creation
and all four `getaddressinfo` calls including the two negative ones.

## Explanation

A Bitcoin Core node can have several wallets loaded at once — mine had three, including
the unnamed `""` wallet Polar creates. RPCs split into two groups. Node-wide calls like
`getblockcount` or `getrawmempool` describe the chain, which is the same regardless of
which wallet is asking. Wallet-scoped calls like `getnewaddress`, `getbalances`,
`listunspent` and `sendtoaddress` are questions about one specific keychain, and the
node cannot guess which one I mean.

`-rpcwallet=<name>` supplies that missing piece. Without it, a node with multiple wallets
loaded returns an error rather than picking one, which is the safe behaviour.

The consequence of getting it wrong is that the command still succeeds — it just answers
about the wrong keychain. `getnewaddress` in the wrong context hands out an address whose
private key lives in a wallet I did not intend, so money sent there is not spendable by
the wallet I thought I was funding. `getbalances` in the wrong context reports someone
else's money. My cross-check above is the concrete version of this: the same string
`bcrt1qfsw0f…hx0q` is "mine" to `miner` and "not mine" to `receiver`. Ownership is not a
property of the address, it is a property of the relationship between an address and a
particular wallet's keys.
