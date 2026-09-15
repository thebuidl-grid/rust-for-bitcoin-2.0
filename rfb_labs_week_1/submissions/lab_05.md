# Lab 05 — Broadcast and mempool

<!-- Replace every TODO line. The grader scores a section 0 while a TODO remains in it. Rewrite the Explanation in your own words. -->

## Commands used

```bash
# Send 1 BTC and do NOT mine afterwards.
bitcoin-cli -rpcwallet=miner sendtoaddress <classmate-address> 1

# The node's local mempool should now contain that TXID.
bitcoin-cli getrawmempool

# The sender's view: zero confirmations, no block hash.
bitcoin-cli -rpcwallet=miner gettransaction <txid>

# The receiver's view: value visible, but only as untrusted-pending.
bitcoin-cli -rpcwallet=receiver getbalances
```

Tests:

```bash
cargo test --test lab_05
```

`observe_unconfirmed_payment` runs these four calls in order and reports whether the
returned TXID is present in the mempool list.

## Terminal output

The payment, and the TXID Bitcoin Core returned for it:

```
$ bitcoin-cli -rpcwallet=miner sendtoaddress bcrt1q0mfhzwfrmz5y88fnvm8k6sdysemr9yd8qwznu7 1
335c3feb471f8a50b354b8a4717fd53c81162922442fb3aef197de6ab5018d70
```

That same TXID, now in the node's mempool:

```
$ bitcoin-cli getrawmempool
[
  "335c3feb471f8a50b354b8a4717fd53c81162922442fb3aef197de6ab5018d70"
]
```

The sender's view. No block has been mined, so no mining took place between the two
calls:

```
$ bitcoin-cli -rpcwallet=miner gettransaction 335c3feb471f8a50b354b8a4717fd53c81162922442fb3aef197de6ab5018d70
{
  "amount": -1.00000000,
  "fee": -0.00002820,
  "confirmations": 0,
  "trusted": true,
  "txid": "335c3feb471f8a50b354b8a4717fd53c81162922442fb3aef197de6ab5018d70",
  "details": [
    {
      "address": "bcrt1q0mfhzwfrmz5y88fnvm8k6sdysemr9yd8qwznu7",
      "category": "send",
      "amount": -1.00000000,
      "vout": 1,
      "fee": -0.00002820,
      "abandoned": false
    }
  ],
  "lastprocessedblock": {
    "hash": "3d3ef2fb461a5e1797afc3e087bc4916497de34df3c3ba465fd7eb9b73303604",
    "height": 201
  }
}
```

`confirmations` is `0`, and there is **no `blockhash` field at all** — the field is
omitted rather than set to null, because there is no block to name. `amount` is
negative because this is money leaving the wallet, and the fee of `0.00002820` BTC is
reported separately from the 1 BTC sent. `lastprocessedblock` still reads height 201,
the tip before the payment, confirming nothing was mined.

The receiver's view of the same payment:

```
$ bitcoin-cli -rpcwallet=receiver getbalances
{
  "mine": {
    "trusted": 0.00000000,
    "untrusted_pending": 1.00000000,
    "immature": 0.00000000
  },
  "lastprocessedblock": {
    "hash": "3d3ef2fb461a5e1797afc3e087bc4916497de34df3c3ba465fd7eb9b73303604",
    "height": 201
  }
}
```

The 1 BTC is visible but sits entirely in `untrusted_pending`, with `trusted` at
zero. The receiver's wallet can see the money and refuses to count it. That gap
between "visible" and "spendable" is the whole lab: the payment exists, every node
that heard it agrees it exists, and none of that makes it settled.

Tests:

```
$ cargo test --test lab_05
running 4 tests
test reads_local_mempool_txids ... ok
test sends_payment_in_sender_wallet_context ... ok
test reads_wallet_transaction_status ... ok
test observes_broadcast_without_confirmation ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

![Unconfirmed payment in the mempool](evidence/lab05-unconfirmed-payment.png)

A still taken from a screen recording of the `backend1` node terminal, captured
while the payment was still unconfirmed. Reading down the frame: the sender's
`gettransaction` details with `"category": "send"`, `"amount": -1.00000000` and
`"fee": -0.00002820`; then the receiver's `getbalances` showing
`"trusted": 0.00000000` against `"untrusted_pending": 1.00000000`; and both reporting
`lastprocessedblock` at height 201, proving no block was mined between them.

The frame also shows the start of the verbosity-2 decode used in Lab 06, where the
input jumps straight from `scriptSig` to `txinwitness` with no `prevout` — the
mempool behaviour described in that lab.

## Explanation

A payment passes through four distinct states, and the lab freezes it in the third.

**Built and signed.** The wallet selects UTXOs, constructs inputs and outputs, and
signs. At this point the transaction exists only in memory on one machine. Nobody
else knows about it and nothing has moved.

**Broadcast.** The transaction is relayed to peers. Each receiving node performs its
own validation: are the inputs real and unspent, do the signatures verify, does the
fee meet its relay minimum. Broadcast is a *request*, not an outcome.

**Mempool.** Nodes that accepted it hold it in memory as a valid but unconfirmed
transaction, waiting for a miner to include it. Two things about this state matter.
The mempool is **per-node** — it is local memory, not consensus. Different nodes
hold slightly different mempools, and a node restart empties it. And the state is
**reversible**: the transaction can be evicted for low fees, dropped after a
timeout, or replaced by a conflicting spend of the same inputs. Nothing is settled.

**Confirmed.** A miner includes it in a block and the network accepts that block.
Only now does it become part of the agreed history, and only now do the inputs
count as spent chain-wide.

The evidence shows the gap precisely. The TXID exists and both wallets can see the
transaction, yet `confirmations` is 0 and there is no `blockhash` because no block
contains it. The receiver's funds land in `untrusted_pending` rather than `trusted`
— Bitcoin Core's own vocabulary for "I can see this, and I will not treat it as
money yet."

**Broadcast is not confirmation.** Having a TXID only proves a transaction was
constructed and relayed. Until it is in a block, the sender can potentially replace
it with a conflicting transaction spending the same inputs, and any node may drop
it. This is why accepting zero-confirmation payments for anything valuable is
unsafe: the receiver has a promise, not a settlement.
