# Lab 04 — UTXOs and outpoints

## Commands used

The following commands were used to inspect the UTXOs and calculate balances:

1. **Listing all unspent transaction outputs**:
   ```bash
   bitcoin-cli -rpcwallet=miner listunspent
   ```

2. **Inspecting wallet balances to reconcile the sum**:
   ```bash
   bitcoin-cli -rpcwallet=miner getbalances
   ```

3. **Running the unit tests**:
   ```bash
   cargo test --test lab_04
   ```

---

## Terminal output

### 1. Verification of the Rust implementation:
Running `cargo test --test lab_04` returns:
```text
running 4 tests
test constructs_unique_outpoint ... ok
test decodes_listunspent_response ... ok
test sums_only_spendable_outputs ... ok
test selects_most_confirmed_spendable_utxo ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 2. Sample output of a spendable UTXO from `listunspent`:
```json
[
  {
    "txid": "7ac16089307ff036c0a6b7d2db6b2e1f4094191bb819bc25501fb08cd3c13e51",
    "vout": 0,
    "address": "bcrt1qmineraddress...",
    "label": "mining",
    "scriptPubKey": "0014798993f3dcdb31495e2634e00787a2d1d441dbf7",
    "amount": 50.00000000,
    "confirmations": 101,
    "spendable": true,
    "solvable": true,
    "desc": "wpkh([6bb8f05c/0'/0'/0']033a0...#9lvd88m0)",
    "safe": true
  }
]
```

---

## Evidence references

- Code is implemented in [lab04_utxos.rs](file:///home/dorine/Music/rust-for-bitcoin-2.0/rfb_labs_week_1/src/labs/lab04_utxos.rs).
- Successful test execution verifies correct parsing of the raw JSON array containing unspent outputs and correct calculation of sum and selection.

---

## Explanation

- **UTXO (Unspent Transaction Output)**: In Bitcoin, coins do not exist as a numeric balance field inside an account (unlike traditional banks or accounts-based systems like Ethereum). Instead, bitcoins exist as discrete chunks of value called UTXOs. Every transaction consumes one or more existing UTXOs (inputs) and creates one or more new UTXOs (outputs).
- **OutPoint**: A unique identifier for a specific UTXO. It consists of the transaction ID (`txid`) that created the output, and the index (`vout`) of that output within the transaction (0-indexed). Thus, `txid:vout` uniquely references a single UTXO in the entire history of the blockchain.
- **Why a wallet balance is their sum**: A Bitcoin wallet does not store a "balance" value directly. Instead, a wallet is a collection of private keys. To determine its balance, the wallet software scans the blockchain database for all UTXOs that can be unlocked/spent by the private keys it controls. The wallet balance is simply the sum of the amounts of all these individual spendable UTXOs. It is a calculated view, not a database entry.
