# Lab 08 — Block security

## Commands used

**Rust commands:**
```bash
cargo test lab_08
cargo run --example lab08_demo
```

**Bitcoin Core commands (via Polar):**
```bash
# Inspect block header
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass \
  getblockheader "5b56c83ecac6ae44f9b5369a954fcb49fc8c06d78f1096bb4f133ec1e5a4be36"

# Check transaction confirmations
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass \
  -rpcwallet=receiver gettransaction "9b33b32b1a605548bcf3fa4061d08d2dc96bc6906dca84b2f53effbe84150a8b"

# Mine additional blocks
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass \
  generatetoaddress 5 "bcrt1qslj8q83p3p6ugf8ecdhknpgcl8ntrec4eet0v6"
```

## Terminal output

**Initial block header inspection:**
```
Block Header Evidence:
  Hash:              5b56c83ecac6ae44f9b5369a954fcb49fc8c06d78f1096bb4f133ec1e5a4be36
  Height:            712
  Previous Hash:     551ca490ff04dacf69191345d584e9606c1b77a8cb36ce9e2d810612484f63b1
  Merkle Root:       0078de98f4b923323a5a309a5b600fea65aedc830df2912cc948cfed78b211cf
  Nonce:             0
  Difficulty:        0.00000000046565423739069247
  Bits:              207fffff
  Confirmations:     1
  Chainwork:         0000000000000000000000000000000000000000000000000000000000000592
```

**Confirmation progression:**
```
Initial Confirmations: 1
[Mining 5 additional blocks...]
Final Confirmations:   6
```

**Security Report:**
```
Transaction Security:
  TXID:                  9b33b32b1a605548bcf3fa4061d08d2dc96bc6906dca84b2f53effbe84150a8b
  Block Hash:            5b56c83ecac6ae44f9b5369a954fcb49fc8c06d78f1096bb4f133ec1e5a4be36
  Block Height:          712
  Confirmations Before:  1
  Confirmations After:   6
```

## Evidence references

![Lab 08 Demo Output](examples/lab_08_demo.rs)


## Explanation

**Hash Links:**
Each block header contains the hash of the previous block (`previousblockhash`), creating an immutable chain. Changing any past block would break the link, making tampering evident.

**Merkle Root:**
The Merkle root (`0078de98f4b923...`) is a cryptographic commitment to all transactions in the block. It's computed by hashing pairs of transaction IDs in a tree structure, ensuring no transaction can be added, removed, or modified without changing the Merkle root.

**Proof-of-Work:**
Miners search for a nonce value that produces a block hash below the target (represented by `bits: 207fffff` and `difficulty: 0.000000000465`). The nonce of 0 in regtest indicates minimal difficulty, but on mainnet this requires massive computational work.

**Confirmation Depth:**
Confirmations measure how many blocks have been mined on top of a transaction's block. Going from 1→6 confirmations means 5 additional blocks were mined. Each additional block exponentially increases the cost to reorganize (reverse) the transaction, as an attacker would need to re-mine all those blocks with greater accumulated work (`chainwork`).
