# Lab 04 — UTXOs and outpoints

## Commands used

The command used was:
` btc -rpcwallet=miner listunspent`

## Terminal output
Terminal Output:

```
    ─(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_1]
└─$ btc -rpcwallet=miner listunspent
[
  {
    "txid": "39778cb2dd37cc26dcd7c579aac57c9b1a9cd23e68f693564f8a7e46a1afec7b",
    "vout": 0,
    "address": "bcrt1qak6v6st6wqakcnfp7h5q9vmlkz8fs4wc8wlweh",
    "label": "miner-address",
    "scriptPubKey": "0014edb4cd417a703b6c4d21f5e802b37fb08e9855d8",
    "amount": 12.50000000,
    "confirmations": 101,
    "spendable": true,
    "solvable": true,
    "desc": "wpkh([152ffe34/84h/1h/0h/0/2]027364fc83502ceb92c9201132a7aff61f512941f28ecf2d9497a578ade91e69d4)#lqrf4t3y",
    "parent_descs": [
      "wpkh([152ffe34/84h/1h/0h]tpubDCeqpu7Vo6oGHj1gciL8iCHtADRfXdhAoThC68vk9x936Yd314aiWuX9CW9vG5qNAmjE79Z8qL1HwKaoHsbgcXnXoYacrHvwLknyokkEaix/0/*)#mdnalc4u"
    ],
    "safe": true
  }
]

```

## Evidence references
The screenshot for the command ran:

![ProjectScreenshot](evidence/Lab%204.png)

## Explanation

A **UTXO** (Unspent Transaction Output) is an output from a past
transaction that hasn't been spent yet — it represents a specific chunk
of bitcoin sitting at a specific address, available to be used as an
input in a future transaction. Bitcoin doesn't track "account balances"
the way a bank does; instead, the entire ledger is just a record of which
outputs exist and haven't yet been consumed. When one spends a bitcoin,
he/she is not decrementing a number — they're consuming one or more existing
UTXOs entirely as inputs and creating new outputs (a payment to the
recipient, and usually a "change" output back to oneself for whatever
wasn't spent).

An **outpoint** is simply the unique identifier for a UTXO: the pairing
of a transaction ID (`txid`) and the specific output index within that
transaction (`vout`), since a single transaction can create multiple
outputs. Together, `txid:vout` unambiguously points to one specific
output — which is exactly what my `OutPoint` struct captures, and what
gets referenced when a later transaction spends that output as an input.

A wallet's balance is the **sum of every UTXO it currently controls and
can spend** — which is exactly what `sum_spendable_utxos` calculates. In
the terminal output above, `listunspent` returned exactly one UTXO
(12.5 BTC, the matured coinbase reward from Lab 03), and the wallet's
`getbalances` trusted balance matches that amount precisely — because
with only one spendable UTXO, the "balance" and the "value of that single
UTXO" are the same number. As a wallet accumulates more UTXOs over time
(from mining, receiving payments, or change outputs from its own spends),
its reported balance becomes the sum across all of them.

This UTXO model is also why Bitcoin Core distinguishes `spendable` from
non-spendable outputs — an immature coinbase output, for example, exists
as a real output in the chain but is excluded from the spendable sum
until it matures (exactly what Lab 03 demonstrated), which is why my
`sum_spendable_utxos` function filters on the `spendable` flag before
summing rather than summing every UTXO indiscriminately.
