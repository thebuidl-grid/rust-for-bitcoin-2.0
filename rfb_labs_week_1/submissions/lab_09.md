# Lab 09 — Multi-UTXO coin selection

## Commands used

Rust:

```
cargo test --test lab_09
cargo fmt --check
cargo run --example lab09
```

`examples/lab09.rs` funds Alice with three fresh 0.4 BTC UTXOs (via `create_three_funding_transactions`),
confirms them, has her send 1 BTC, mines that spend, then decodes and audits it. Note:
`audit_multi_utxo_spend` itself sends and decodes back-to-back (matching its test's mocked call
sequence), which hits the same real-node limitation documented in Lab 06 —
`getrawtransaction` verbosity 2 can't fill in `prevout` for a still-unconfirmed transaction. Calling
`audit_multi_utxo_spend` directly against the live node reproduces that failure
(`MissingField("prevout")`), shown below. The example works around it by running the same
underlying steps manually with a mining step inserted in between, which succeeds.

Bitcoin Core RPCs (run directly in Polar's node terminal):

```
bitcoin-cli createwallet alice
bitcoin-cli -rpcwallet=alice getnewaddress coin-selection
bitcoin-cli -rpcwallet=miner sendtoaddress <alice-address> 0.4   # x3
bitcoin-cli generatetoaddress 1 $MINER_ADDR
bitcoin-cli -rpcwallet=alice listunspent
bitcoin-cli -rpcwallet=alice sendtoaddress <new-receiver-address> 1
bitcoin-cli generatetoaddress 1 $MINER_ADDR
bitcoin-cli getrawtransaction <spend-txid> 2
```

## Terminal output

`cargo test --test lab_09`:

```
running 4 tests
test creates_three_separate_funding_transactions ... ok
test filters_confirmed_utxos_for_alice_address ... ok
test sends_one_btc_from_alice ... ok
test audits_three_input_spend_payment_change_and_fee ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Calling `audit_multi_utxo_spend` directly against the real node (fails as explained above,
reproducible, not a logic bug):

```
thread 'main' panicked at examples/lab09.rs:44:10:
audit_multi_utxo_spend failed: MissingField("prevout")
```

`cargo run --example lab09` (real node, full send → mine → decode cycle, via the completed Rust
implementation):

```
funding txids: [
    "001d67f3b2d65b31937414b29906c1482ba1dc511f187d8e6ec4d7565d977bc6",
    "7cd61cc385dd1059a1597eea67d4973020e499ad52a9c8db5741bbfa1efafafc",
    "f36b86f2ad224d417dd97d5c6e9f9ac6e729fcb6894b834c59793394d4c87b67",
]
Alice's confirmed UTXOs: 3
MultiUtxoAudit {
    funding_outpoints: [
        OutPoint { txid: "7cd61cc385dd1059a1597eea67d4973020e499ad52a9c8db5741bbfa1efafafc", vout: 0 },
        OutPoint { txid: "f36b86f2ad224d417dd97d5c6e9f9ac6e729fcb6894b834c59793394d4c87b67", vout: 1 },
        OutPoint { txid: "001d67f3b2d65b31937414b29906c1482ba1dc511f187d8e6ec4d7565d977bc6", vout: 1 },
    ],
    spend_txid: "13edf9e09696624e60535b569f72609430a7bcd448eb702535a51bd561c84bc7",
    spend_input_count: 3,
    payment_and_change: PaymentAndChange {
        payment: DecodedOutput { vout: 1, value: 1.0, address: Some("bcrt1qxlgnqhcvf6qhul8ydkqv7nhywmp36qfrxrsa6e"), .. },
        change: Some(DecodedOutput { vout: 0, value: 0.1999448, address: Some("bcrt1qhjvvzrf85ke8nnxjkydqtcudqwlljwcrksw8wj"), .. }),
    },
    fee: 0.0000552,
}
```

Raw `bitcoin-cli` output from the manual walkthrough (a separate, independent run — 3 funding
transactions of 0.4 BTC each to Alice's `bcrt1q3tq78...` address, all confirmed):

```
$ bitcoin-cli -rpcwallet=alice listunspent
[
  { "txid": "e7d061a4...", "amount": 0.4, "confirmations": 1, "spendable": true, ... },
  { "txid": "a2d72135...", "amount": 0.4, "confirmations": 1, "spendable": true, ... },
  { "txid": "090d185b...", "amount": 0.4, "confirmations": 1, "spendable": true, ... }
]

$ bitcoin-cli -rpcwallet=alice sendtoaddress <receiver> 1
9e6274ca...

$ bitcoin-cli getrawtransaction 9e6274ca... 2
{
  "vin": [
    { "txid": "a2d72135...", "vout": 1, "prevout": { "value": 0.4 } },
    { "txid": "e7d061a4...", "vout": 0, "prevout": { "value": 0.4 } },
    { "txid": "090d185b...", "vout": 1, "prevout": { "value": 0.4 } }
  ],
  "vout": [
    { "value": 0.1999448, "n": 0, "scriptPubKey": { "address": "bcrt1q8fjk0..." } },
    { "value": 1.0,       "n": 1, "scriptPubKey": { "address": "bcrt1qshmxp..." } }
  ],
  "fee": 0.0000552
}
```

Both the Rust implementation and the raw RPC data agree: Alice's spend required all **3** of her
funding UTXOs (0.4 × 3 = 1.2 BTC), which were fully consumed and split into the 1 BTC payment plus
0.1999448 BTC change, with the tiny remainder (0.0000552 BTC) as the fee — `1.2 = 1.0 + 0.1999448 +
0.0000552`.

## Evidence references

Terminal output above was captured directly from Polar's node terminal and from
`cargo run --example lab09`; no separate screenshots were taken for this lab.

## Explanation

A wallet can only spend whole UTXOs, never a fraction of one. Alice needed to pay 1 BTC, but her
largest single UTXO was only 0.4 BTC — no single input could cover the payment, so Bitcoin Core's
coin selection had to reach for more than one, and in this case ended up using all three 0.4 BTC
UTXOs (combined 1.2 BTC) to have enough. Whatever wasn't needed for the payment came back as a new
change output under Alice's own control, and the tiny leftover became the miner fee — exactly the
same value-conservation rule from Lab 06, just now with three inputs feeding one transaction
instead of one.

The privacy trade-off: by combining all three UTXOs into a single transaction, this spend publicly
reveals — to anyone reading the blockchain — that all three of those previously-separate UTXOs are
controlled by the *same* wallet/entity. Before this transaction, an outside observer could only see
three unrelated-looking payments to three addresses; they'd have no particular reason to assume the
same person controlled all three. The moment they're spent together as inputs to one transaction,
that assumption becomes a near-certainty (since Bitcoin Core wouldn't sign for keys it doesn't
control), a technique commonly called "common-input-ownership" analysis. This is exactly why privacy-
conscious wallets try to avoid unnecessarily combining UTXOs, and why receiving many small payments
to the same address (or even the same wallet) can quietly erode privacy the first time you need to
spend enough of them together to cover a larger payment.
