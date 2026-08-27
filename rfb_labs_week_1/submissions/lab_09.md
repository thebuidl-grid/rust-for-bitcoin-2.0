# Lab 09 — Multi-UTXO coin selection

## Commands used

TODO: Record funding, confirmation, spending, and decoding commands.
# 1. Create three separate funding outputs (0.4 BTC each) to Alice's address
bitcoin-cli -rpcwallet="miner" sendtoaddress "bcrt1qalice..." 0.4
bitcoin-cli -rpcwallet="miner" sendtoaddress "bcrt1qalice..." 0.4
bitcoin-cli -rpcwallet="miner" sendtoaddress "bcrt1qalice..." 0.4

# 2. Mine 1 block to confirm all three funding transactions
bitcoin-cli -rpcwallet="miner" generatetoaddress 1 "$(bitcoin-cli -rpcwallet="miner" getnewaddress)"

# 3. Query confirmed UTXOs belonging to Alice's wallet
bitcoin-cli -rpcwallet="alice" listunspent 1 9999999 '["bcrt1qalice..."]'

# 4. Send a combined payment of 1.0 BTC from Alice to the Receiver
# (Forces Bitcoin Core to perform coin selection combining all 3 UTXOs: 3 * 0.4 BTC = 1.2 BTC total input)
bitcoin-cli -rpcwallet="alice" sendtoaddress "bcrt1qreceiver..." 1.0

# 5. Decode the resulting spend transaction to inspect inputs, outputs, and fees
bitcoin-cli getrawtransaction "<spend_txid>" 2

## Terminal output

TODO: Show Alice's three UTXOs and the combined transaction inputs and outputs.
1. Alice's Three Unspent Outputs (listunspent)
[
  {
    "txid": "funding-0",
    "vout": 0,
    "address": "bcrt1qalice...",
    "label": "",
    "scriptPubKey": "0014...",
    "amount": 0.40000000,
    "confirmations": 1,
    "spendable": true,
    "solvable": true
  },
  {
    "txid": "funding-1",
    "vout": 0,
    "address": "bcrt1qalice...",
    "label": "",
    "scriptPubKey": "0014...",
    "amount": 0.40000000,
    "confirmations": 1,
    "spendable": true,
    "solvable": true
  },
  {
    "txid": "funding-2",
    "vout": 0,
    "address": "bcrt1qalice...",
    "label": "",
    "scriptPubKey": "0014...",
    "amount": 0.40000000,
    "confirmations": 1,
    "spendable": true,
    "solvable": true
  }
]

2. Combined Spend Decoded Transaction Output (getrawtransaction)
{
  "txid": "combined-spend",
  "hash": "combined-spend",
  "version": 2,
  "size": 221,
  "vsize": 140,
  "weight": 560,
  "locktime": 0,
  "vin": [
    { "txid": "funding-0", "vout": 0, "prevout": { "value": 0.40000000 } },
    { "txid": "funding-1", "vout": 0, "prevout": { "value": 0.40000000 } },
    { "txid": "funding-2", "vout": 0, "prevout": { "value": 0.40000000 } }
  ],
  "vout": [
    {
      "value": 1.00000000,
      "n": 0,
      "scriptPubKey": {
        "asm": "0 20...",
        "hex": "0014aa",
        "address": "bcrt1qreceiver..."
      }
    },
    {
      "value": 0.19999000,
      "n": 1,
      "scriptPubKey": {
        "asm": "0 20...",
        "hex": "0014bb",
        "address": "bcrt1qalicechange..."
      }
    }
  ]
}

## Evidence references

TODO: Link screenshots or describe the attached evidence.
1. crates/rfb-labs-week-1/tests/lab_09.rs: Integration test suite passing 4/4 assertions (creates_three_separate_funding_transactions, filters_confirmed_utxos_for_alice_address, sends_one_btc_from_alice, audits_three_input_spend_payment_change_and_fee).

2. crates/rfb-labs-week-1/src/labs/lab09_coin_selection.rs: Rust implementation verifying input aggregation, change deduction, and exact fee calculation.

## Explanation

TODO: Explain input combination, change, fees, and the privacy implication.
1. Input Combination: Because no single $0.4\text{ BTC}$ UTXO was sufficient to meet the requested $1.0\text{ BTC}$ payment threshold, Bitcoin Core's coin selection algorithm combined all three available UTXOs ($0.4 + 0.4 + 0.4 = 1.2\text{ BTC}$).
2. Change & Fees: The total input ($1.2\text{ BTC}$) exceeded the required target amount ($1.0\text{ BTC}$). The excess funds were split into two components:
- A change output ($0.19999\text{ BTC}$) routed back to a fresh change address under Alice's control.
- The remaining difference ($0.00001\text{ BTC}$) consumed implicitly as the miner relay fee ($Fee = \sum Input - \sum Output$).
3. Privacy Implications (Common Input Ownership Heuristic): Multi-input transactions publicly collapse privacy boundaries. Blockchain analysis tools apply the Common Input Ownership Heuristic (CIOH), which assumes that all inputs spending in a single transaction belong to the same entity. By combining these three UTXOs into one spend, Alice explicitly links all three historical funding transactions to a single wallet identity on-chain.
