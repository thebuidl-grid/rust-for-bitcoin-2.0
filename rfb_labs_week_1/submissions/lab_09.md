# Lab 09 — Multi-UTXO coin selection

## Commands used

Rust: `cargo run --example run` (calls the following in sequence):

- `sendtoaddress <alice_address> 0.4` (wallet: miner) — x3, creating three separate funding transactions
- `generatetoaddress 1 <miner_address>` — confirm the three funding transactions
- `listunspent` (wallet: alice) — via `confirmed_utxos_for_address`, filtered to Alice's address
- `sendtoaddress <receiver_address> 1` (wallet: alice) — the combined spend
- `getrawtransaction <spend_txid> 2` — decode the spend to verify multi-input selection, via `audit_multi_utxo_spend`

## Terminal output

=== Lab 09: coin selection ===
funding txids: ["75a1d56440e25d895fec131d6b84721bc5183b513bda8017d1a873cd2d70201e", "176817a63327ac5c7b50f0bddba1e5a004271e83e260dd8a69e914f81ab747da", "a297c479ca6d39a54b3539d546429f2619628aab9a5075b14012ff942e96535c"]
alice's confirmed UTXOs:
a297c479ca6d39a54b3539d546429f2619628aab9a5075b14012ff942e96535c:1 amount 0.4 confirmations 1
176817a63327ac5c7b50f0bddba1e5a004271e83e260dd8a69e914f81ab747da:0 amount 0.4 confirmations 1
75a1d56440e25d895fec131d6b84721bc5183b513bda8017d1a873cd2d70201e:1 amount 0.4 confirmations 1
MultiUtxoAudit {
funding_outpoints: [
OutPoint { txid: "a297c479ca6d39a54b3539d546429f2619628aab9a5075b14012ff942e96535c", vout: 1 },
OutPoint { txid: "176817a63327ac5c7b50f0bddba1e5a004271e83e260dd8a69e914f81ab747da", vout: 0 },
OutPoint { txid: "75a1d56440e25d895fec131d6b84721bc5183b513bda8017d1a873cd2d70201e", vout: 1 },
],
spend_txid: "6fd38839384543ce2854e6317cc197e2f11c9c7c5b19fa0628d41f6d838ad500",
spend_input_count: 3,
payment_and_change: PaymentAndChange {
payment: DecodedOutput {
vout: 1,
value: 1.0,
address: Some("bcrt1qd0jasulp7k5xx3yrrg4s2jelnu526dj47qmskq"),
script_pub_key_hex: "00146be5d873e1f5a86344831a2b054b3f9f28ad3655",
},
change: Some(
DecodedOutput {
vout: 0,
value: 0.1999448,
address: Some("bcrt1qlvdyyxcttdzg7hqs88tqpf2xjy76y3nkks3jgm"),
script_pub_key_hex: "0014fb1a421b0b5b448f5c1039d608a546913da24676",
},
),
},
fee: 5.52e-5,
}

## Evidence references

Screenshot: `evidence/lab09.png`

## Explanation

Alice received three separate 0.4 BTC payments, each its own distinct transaction and therefore its own distinct UTXO — three coins of 0.4 BTC each (totaling 1.2 BTC), not one combined balance sitting anywhere as a single number. My output shows all three, each with a different txid, sitting confirmed at 1 confirmation each.

When Alice tried to send 1 BTC, no single one of her 0.4 BTC UTXOs was large enough on its own, so Bitcoin Core's coin selection had to combine multiple inputs into one transaction — confirmed directly by `spend_input_count: 3`, meaning all three of Alice's UTXOs were consumed to fund this single payment. Each is spent completely; there's no partial spending of a UTXO. Since the combined input value (1.2 BTC) exceeds the 1 BTC payment, the excess minus the fee (1.2 − 1.0 − 0.0000552 = 0.1999448) came back as a change output to a brand-new address Alice's wallet controls, rather than being left over anywhere.

The privacy implication: combining multiple UTXOs into one transaction reveals to any outside observer that all of the spent inputs' addresses are controlled by the same entity — there'd be no reason for one transaction to spend unrelated people's coins together. This is a known blockchain analysis technique (the common-input-ownership heuristic). Someone who only knew about one of Alice's three original 0.4 BTC addresses can now link it to the other two the moment she combines them in a single spend, even though nothing else about her identity was ever directly revealed.
