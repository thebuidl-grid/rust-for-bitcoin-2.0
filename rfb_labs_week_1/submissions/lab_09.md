# Lab 09 — Multi-UTXO coin selection

## Commands used

```bash
cargo run -- lab09
```

```bash
bitcoin-cli ... createwallet alice
bitcoin-cli ... -rpcwallet=alice    getnewaddress funding
bitcoin-cli ... -rpcwallet=receiver getnewaddress alice-payment

bitcoin-cli ... -rpcwallet=miner sendtoaddress bcrt1qvlx7kyhqy2xlhhfmhcj2v8gzjyzsak6vcrpnps 0.4
bitcoin-cli ... -rpcwallet=miner sendtoaddress bcrt1qvlx7kyhqy2xlhhfmhcj2v8gzjyzsak6vcrpnps 0.4
bitcoin-cli ... -rpcwallet=miner sendtoaddress bcrt1qvlx7kyhqy2xlhhfmhcj2v8gzjyzsak6vcrpnps 0.4

bitcoin-cli ... generatetoaddress 1 bcrt1qfsw0fvcdjruj7d746sxqy0nnnpptcvsyslhx0q
bitcoin-cli ... -rpcwallet=alice listunspent

bitcoin-cli ... -rpcwallet=alice sendtoaddress bcrt1qkk8hsu3r2pxwkmkjp6gcplr9qurrzdp50nxzjc 1
bitcoin-cli ... getrawtransaction 92c0eb2f218550091f6ca1c4b9769e83a8cf4f6e57a8f1d20d14d84994643113 2
```

Three separate 0.4 BTC sends, not one 1.2 BTC send. That is what forces Alice to hold
three distinct UTXOs, none of which alone can cover a 1 BTC payment.

## Terminal output

Funding TXIDs:

```json
[
  "0b25873cd463a702fa616f0b176229b3079294623726ac0b631161ea01e89514",
  "9f82aeee0e5754be331f61ab60c8672838250fbc69c30834dfc66ceebf2c0bc5",
  "9dca7f96e6b26307adfd87c0dcbf175eb3b9c99e77b7cd9c525b54e5d140f6c3"
]
```

After confirming, Alice's wallet holds three separate outputs, all paying the same
address, each 0.4 BTC with 1 confirmation:

```text
9dca7f96...f6c3:1   0.40000000   bcrt1qvlx7kyhqy2xlhhfmhcj2v8gzjyzsak6vcrpnps
9f82aeee...0bc5:1   0.40000000   bcrt1qvlx7kyhqy2xlhhfmhcj2v8gzjyzsak6vcrpnps
0b25873c...9514:1   0.40000000   bcrt1qvlx7kyhqy2xlhhfmhcj2v8gzjyzsak6vcrpnps

distinct UTXO count: 3
```

The audited spend:

```json
{
  "spend_txid": "92c0eb2f218550091f6ca1c4b9769e83a8cf4f6e57a8f1d20d14d84994643113",
  "spend_input_count": 3,
  "funding_outpoints": [
    { "txid": "9dca7f96...f6c3", "vout": 1 },
    { "txid": "9f82aeee...0bc5", "vout": 1 },
    { "txid": "0b25873c...9514", "vout": 1 }
  ],
  "payment_and_change": {
    "payment": { "vout": 1, "value": 1.0,       "address": "bcrt1qkk8hsu3r2pxwkmkjp6gcplr9qurrzdp50nxzjc" },
    "change":  { "vout": 0, "value": 0.1999448, "address": "bcrt1qgnte49nmwppc9e3c8stepcam0xzmmw7yqnvksw" }
  },
  "fee": 0.0000552
}

inputs required: 3
payment: 1 BTC
change:  0.1999448 BTC
fee:     0.0000552 BTC
```

Every required point is here. Three inputs were needed, not one. The three inputs are
exactly the three funding outpoints, each consumed in full at 0.4 BTC. The receiver got
precisely 1 BTC. The surplus came back as 0.1999448 BTC of change. And the balance:

```text
1.2 = 1 + 0.1999448 + 0.0000552
```

Note the change is at `vout 0` and the payment at `vout 1` — Bitcoin Core shuffles output
order, so position carries no meaning.

## Evidence references

Full run log at `evidence/week1-labs-01-09.log`, lines 851-1250, including all three
`listunspent` entries and the complete verbose decode of the combined spend.

## Explanation

**Why more than one input.** Inputs are references to whole UTXOs, and an input spends
its output entirely — there is no way to spend 0.6 of a 0.4 BTC output, and no way to
spend part of one. Alice's largest single UTXO was 0.4 BTC, so covering a 1 BTC payment
plus fee required combining three of them: 0.4 + 0.4 + 0.4 = 1.2 BTC of input value. Two
would have given only 0.8, which is short.

**Where the surplus goes.** The transaction brought in 1.2 BTC and assigned 1 BTC to the
receiver. The remaining 0.1999448 BTC returned to Alice at
`bcrt1qgnte49n…vksw`, a change address her wallet generated internally — I never typed it.
The unassigned 0.0000552 BTC is the fee. Note this fee is larger than Lab 06's 0.0000282
for a payment of the same size: three inputs means three sets of signatures and witness
data, so the transaction is physically bigger and costs more at the same rate. Holding
many small UTXOs is genuinely more expensive to spend.

**The privacy trade-off.** This is the part that matters. Before this transaction, the
three 0.4 BTC outputs were separate entries in the UTXO set. Nothing on-chain proved a
single party controlled all three — here they even shared an address, but had they gone to
three different addresses, they would have looked independent.

The moment Alice signed one transaction spending all three, she published a valid
signature for each. Only the holder of the corresponding private key can produce those, so
the transaction is public proof that one entity controls all three outputs. This is the
*common input ownership heuristic*, and it is the single most powerful tool in chain
analysis: any set of inputs combined in one transaction is presumed to share an owner.

The damage propagates backwards and forwards. An observer who previously identified just
one of those three outputs — say it came from a KYC exchange withdrawal — can now link the
other two to the same identity, along with everything that funded them and everything they
subsequently pay. One linkage contaminates the whole cluster.

There is a second leak in the same transaction: change detection. An observer sees 1 BTC
to one address and 0.1999448 BTC to another, and the round number is a strong hint that it
is the intended payment while the awkward remainder is change returning to the sender.
Combined with input clustering, that extends the identified cluster forward to Alice's new
change address too.

The trade-off is unavoidable given the mechanics. Alice needed 1 BTC and her coins came in
0.4 BTC pieces; combining them was the only way to pay. Practical mitigations —
coin control to keep unrelated UTXOs apart, avoiding address reuse, keeping separate
wallets for separate purposes, or CoinJoin — reduce the linkage but cannot eliminate the
fact that spending together proves owning together.
