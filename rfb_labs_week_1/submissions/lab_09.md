# Lab 09 — Multi-UTXO coin selection

## Commands used

```
cargo test --test lab_09

bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "<alice-address>" 0.4
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "<alice-address>" 0.4
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "<alice-address>" 0.4
bitcoin-cli -regtest generatetoaddress 1 "<miner-address>"
bitcoin-cli -regtest -rpcwallet=alice listunspent
bitcoin-cli -regtest -rpcwallet=alice sendtoaddress "<receiver-address>" 1
bitcoin-cli -regtest getrawtransaction "<combined-spend-txid>" 2
```

*RPCs are the ones issued by `create_three_funding_transactions`, `confirmed_utxos_for_address`, `send_combined_payment`, and `audit_multi_utxo_spend` in `src/labs/lab09_coin_selection.rs`, verified against the mocked RPC client in `tests/lab_09.rs`. Run the `bitcoin-cli` lines against your live Polar regtest node (mine at least one block after funding so the UTXOs confirm) to capture the terminal output below.*

## Terminal output

Captured against the live regtest node:

```
$ bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1q5k8k0g8pc4nm30ffz59j2rr5y0tq3nsclfwjmu" 0.4
2f3d264a610f0f76473eb56a69d8500ed3dcb0b051e44dd032a40887265a93bb

$ bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1q5k8k0g8pc4nm30ffz59j2rr5y0tq3nsclfwjmu" 0.4
a41bc3ae676838ea3232ede8cb2f5fc2fe33b117b0c0c80fc9a0c43b2b70ed58

$ bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1q5k8k0g8pc4nm30ffz59j2rr5y0tq3nsclfwjmu" 0.4
65cca4dfe9812bb01a74f02a29f8dadd9ddeb70205926de1f204dd2d4460de94

$ bitcoin-cli -regtest generatetoaddress 1 "bcrt1qtdwur5a220ta9f0lndtdqf45f0kmaplt3xea5l"
[ "57215f15b07ecff63091c8e863960e04f0a6002b90f0323011e72b65247bc104" ]

$ bitcoin-cli -regtest -rpcwallet=alice listunspent
[
  { "txid": "65cca4df...", "vout": 1, "amount": 0.40000000, "confirmations": 1, "spendable": true, ... },
  { "txid": "a41bc3ae...", "vout": 1, "amount": 0.40000000, "confirmations": 1, "spendable": true, ... },
  { "txid": "2f3d264a...", "vout": 1, "amount": 0.40000000, "confirmations": 1, "spendable": true, ... }
]

$ bitcoin-cli -regtest -rpcwallet=alice sendtoaddress "bcrt1qx208aadpjxz7ftargmdy64amhslmycnjll2xxl" 1
fab70bc2e21f2a3d913e849b2cbe09be45ac01dd93be7d2cd23b3e98a8f2f19d

$ bitcoin-cli -regtest getrawtransaction "fab70bc2e21f2a3d913e849b2cbe09be45ac01dd93be7d2cd23b3e98a8f2f19d" 2
{
  "txid": "fab70bc2e21f2a3d913e849b2cbe09be45ac01dd93be7d2cd23b3e98a8f2f19d",
  "vsize": 276,
  "vin": [
    { "txid": "2f3d264a...", "vout": 1, ... },
    { "txid": "a41bc3ae...", "vout": 1, ... },
    { "txid": "65cca4df...", "vout": 1, ... }
  ],
  "vout": [
    { "value": 0.19994480, "n": 0, "scriptPubKey": { "address": "bcrt1qap3rzl2emx439mx7sp3gfrpn4vntc3cx7euhx2", ... } },
    { "value": 1.00000000, "n": 1, "scriptPubKey": { "address": "bcrt1qx208aadpjxz7ftargmdy64amhslmycnjll2xxl", ... } }
  ]
}
```

Alice's wallet naturally selected all three 0.4 BTC UTXOs (three separate transactions can't be split into fewer inputs and still cover a 1 BTC payment: `0.4 × 2 = 0.8 < 1.0`, so all three were required). The spend combined them into one transaction with 3 inputs and 2 outputs: 1 BTC to the receiver, ~0.19994 BTC change back to Alice.

As in Lab 06, this `getrawtransaction` call was made on the *unconfirmed* combined-spend transaction, so its `vin` entries do not carry a `prevout` field either — the same real limitation applies here (`decode_verbose_transaction`, which `audit_multi_utxo_spend` reuses, needs the transaction confirmed first on this node to resolve input values).

Fee: 3 × 0.4 − (0.19994480 + 1.00000000) = 1.2 − 1.19994480 = **0.00005520 BTC**.

## Evidence references

Evidence is the live terminal output above, captured directly via `docker exec bitcoind-lab-a bitcoin-cli ...` against a real regtest node (not a screenshot — this session ran headlessly, no Polar GUI was open).

## Explanation

**Input combination:** a wallet can't split a UTXO to pay a smaller amount — each UTXO must be spent whole. To pay 1 BTC when your money is spread across three separate 0.4 BTC UTXOs, a single UTXO isn't enough, and even two together (0.8 BTC) still fall short. The wallet's coin-selection algorithm has to pull in *all three* inputs to have enough value to cover the 1 BTC payment. That's exactly what happened: the transaction has 3 inputs, one per funding UTXO.

**Change:** the three inputs sum to 1.2 BTC, but only 1 BTC needs to go to the receiver. The excess isn't lost or optionally kept by the miner — the wallet creates a second output, the **change output**, sending the leftover (minus the fee) back to an address it controls. That's the `0.19994480` BTC output back to Alice's own wallet, alongside the `1.00000000` BTC payment.

**Fees:** as in Lab 06, the fee is implicit — inputs minus outputs, here `1.2 − 1.19994480 = 0.00005520 BTC`. Combining more inputs makes a transaction physically larger (more signatures to include), and fees scale with size, which is why this 3-input transaction (`vsize: 276`) has a noticeably bigger fee than Lab 06's single-input transaction (`vsize: 141`).

**Privacy implication:** by combining all three UTXOs into one transaction, this spend publicly reveals on-chain that the same wallet controlled all three inputs — anyone analyzing the blockchain can now link Alice's three separate incoming payments together as belonging to one entity, and can often also identify which output is the change (frequently by amount patterns or address reuse), further linking that new address back to Alice too. This is a well-known real privacy leak in Bitcoin: needing to combine coins to make a payment tends to deanonymize which past payments belonged to the same person.
