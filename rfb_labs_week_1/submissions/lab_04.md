# Lab 04 — UTXOs and outpoints

## Commands used

```bash
cargo run -- lab04
```

```bash
bitcoin-cli ... -rpcwallet=miner listunspent
bitcoin-cli ... -rpcwallet=miner getbalances
```

`select_spendable_utxo` filters to `spendable == true` and takes the one with the most
confirmations, breaking ties on `(txid, vout)` so the choice never depends on the order
Bitcoin Core happens to return. `sum_spendable_utxos` adds the amounts myself, and I
compare that against `getbalances` to reconcile.

## Terminal output

```text
$ bitcoin-cli ... -rpcwallet=miner listunspent
  [
    {
      "txid": "1c62ddb27fc72e5abefbc40dc882296e3772e57ee4df5a3a82e1b78554282c7f",
      "vout": 0,
      "address": "bcrt1qfsw0fvcdjruj7d746sxqy0nnnpptcvsyslhx0q",
      "label": "mining",
      "scriptPubKey": "00144c1cf4b30d90f92f37d5d40c023e739842bc3204",
      "amount": 50.00000000,
      "confirmations": 101,
      "spendable": true,
      "solvable": true,
      "safe": true
    }
  ]
```

The selected UTXO and its outpoint:

```json
{
  "txid": "1c62ddb27fc72e5abefbc40dc882296e3772e57ee4df5a3a82e1b78554282c7f",
  "vout": 0,
  "address": "bcrt1qfsw0fvcdjruj7d746sxqy0nnnpptcvsyslhx0q",
  "script_pub_key": "00144c1cf4b30d90f92f37d5d40c023e739842bc3204",
  "amount": 50.0,
  "confirmations": 101,
  "spendable": true
}

--- OutPoint ---
{ "txid": "1c62ddb2...282c7f", "vout": 0 }
```

Reconciliation:

```text
spendable UTXO count: 1
sum(spendable UTXOs)     = 50 BTC
getbalances.mine.trusted = 50 BTC
difference               = 0 BTC
```

Note that `getbalances` also reports `immature: 5000.00000000`, which `listunspent` does
not return at all — those 100 coinbases are owned but not yet spendable.

## Evidence references

Full run log at `evidence/week1-labs-01-09.log`, lines 326-382, including the complete
`listunspent` entry with descriptor fields and the `getbalances` used for reconciliation.

## Explanation

A UTXO is an unspent transaction output: a specific amount of bitcoin locked by a
specific script, created by one transaction and not yet consumed by another. The outpoint
is its coordinate — the `txid` of the transaction that created it plus the `vout` index
of the output within that transaction. Here that is
`1c62ddb2…282c7f:0`. The pair is unique across the whole chain, which is what lets an
input reference precisely one previous output with no ambiguity.

The locking script `00144c1cf4b3…3204` is a version-0 witness program: `0014` marks a
20-byte P2WPKH, and the remaining bytes are the hash of the public key. The address
`bcrt1qfsw0f…hx0q` is just a human-friendly encoding of that same script, not a separate
thing the protocol knows about.

The part that actually changed how I think about this is the balance. A wallet balance is
not a stored number that gets debited and credited like a bank ledger. Nothing anywhere
records "miner has 50 BTC". What exists is a set of individual outputs scattered across
the chain, and the wallet scans for the ones its keys can unlock and adds them up. My
independent sum matched `getbalances` to the satoshi (difference 0) precisely because
both are doing the same addition over the same set.

This also explains the categories. `trusted: 50` and `immature: 5000` differ not because
of two accounts but because the wallet is applying spendability rules to each output
separately. And it explains why spending is not subtraction — to pay 1 BTC out of a
single 50 BTC output I must consume the whole thing and create a new output for the
remainder, which is what the next labs show.
