# Lab 09 — Multi-UTXO coin selection

## Commands used

**Rust commands:**
```bash
cargo test --test lab_09
cargo run --example lab09_demo
```

**Bitcoin Core commands (via Polar):**
```bash
# Send three separate 0.4 BTC payments to Alice
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass \
  -rpcwallet=miner sendtoaddress "bcrt1qhyzrgs9e459zt2y0lnk9s37evg3ysnlqjnzc32" 0.4

# Confirm funding transactions
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass \
  generatetoaddress 1 "bcrt1q..."

# List Alice's UTXOs
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass \
  -rpcwallet=alice listunspent

# Alice sends 1 BTC (combines multiple UTXOs)
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass \
  -rpcwallet=alice sendtoaddress "bcrt1q4p5pyqd9us6tfgu8lt3jxdyfx3y0h3cz0j862k" 1

# Decode the spend transaction
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass \
  getrawtransaction "638673961334a74b52ef6819e7d9df14524e8b79da164c035559643f97114657" 2
```

## Terminal output

**Alice's three confirmed UTXOs:**
```
Alice has 3 confirmed UTXO(s):
  UTXO 1: 0.4 BTC (af70a87489139e23:1)
  UTXO 2: 0.4 BTC (9b9dd32ab0658449:1)
  UTXO 3: 0.4 BTC (a807241d08809ce8:0)
Total: 1.2 BTC
```

**Multi-UTXO spend analysis:**
```
Spend Transaction: 638673961334a74b52ef6819e7d9df14524e8b79da164c035559643f97114657
Input Count:       3

Funding Outpoints (inputs consumed):
  Input 1: af70a87489139e23:1
  Input 2: 9b9dd32ab0658449:1
  Input 3: a807241d08809ce8:0

Outputs:
  Payment:  1 BTC → bcrt1q4p5pyqd9us6tfgu8lt3jxdyfx3y0h3cz0j862k
  Change:   0.1999448 BTC → bcrt1qnnnscnvy7352untp89eejgn6ymp67d7m3rp2h0

Value Conservation:
  Inputs:  1.2 BTC (3 UTXOs × 0.4 BTC)
  Outputs: 1.1999448 BTC
  Fee:     0.0000552 BTC
```

## Evidence references


![Alice's UTXOs](lab09_screenshots/
alice_utxos.png
alice_balance_after_spend.png)

## Explanation

**Input Combination:**
To send 1 BTC, Alice needed to combine all three 0.4 BTC UTXOs (total 1.2 BTC). Bitcoin Core's coin selection algorithm automatically chose these inputs since none of them alone could cover the payment plus fee.

**Change Output:**
Since inputs (1.2 BTC) exceeded the payment (1.0 BTC) plus fee (0.0000552 BTC), the surplus (0.1999448 BTC) was returned to Alice as a change output to a new address in her wallet.

**Fees:**
The fee (0.0000552 BTC) is the difference between total inputs and total outputs. It compensates miners for including the transaction in a block.

**Privacy Implication:**
When multiple UTXOs are spent together in one transaction, blockchain analysis can apply the **common input ownership heuristic** - assuming all inputs belong to the same entity. In this case, all three funding addresses (af70a874..., 9b9dd32a..., a807241d...) are now publicly linked to Alice, revealing that one person controls all three. This reduces privacy by clustering addresses and enabling tracking of funds across the blockchain.
