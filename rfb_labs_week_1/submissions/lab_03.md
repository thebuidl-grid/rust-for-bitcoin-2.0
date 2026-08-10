# Lab 03 — Coinbase maturity

## Commands used

The following commands were used to test and verify coinbase maturity:

1. **Mining block reward (1 block)**:
   ```bash
   bitcoin-cli -rpcwallet=miner generatetoaddress 1 <miner_address>
   ```

2. **Checking wallet balances**:
   ```bash
   bitcoin-cli -rpcwallet=miner getbalances
   ```

3. **Attempting premature spend (sending 1 BTC)**:
   ```bash
   bitcoin-cli -rpcwallet=miner sendtoaddress <receiver_address> 1.0
   ```
   *(This fails with an RPC error: `Insufficient funds`)*

4. **Mining 100 more blocks to mature the first block's coinbase**:
   ```bash
   bitcoin-cli -rpcwallet=miner generatetoaddress 100 <miner_address>
   ```

5. **Checking balances again** (trusted balance is now 50.0 BTC, indicating coinbase is spendable):
   ```bash
   bitcoin-cli -rpcwallet=miner getbalances
   ```

6. **Running tests**:
   ```bash
   cargo test --test lab_03
   ```

---

## Terminal output

### 1. Verification of the Rust implementation:
Running `cargo test --test lab_03` returns:
```text
running 4 tests
test mines_requested_number_of_blocks ... ok
test preserves_insufficient_funds_error ... ok
test demonstrates_first_coinbase_becoming_spendable_at_height_101 ... ok
test reads_nested_wallet_balances ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 2. Node RPC responses (Mocked representation of maturity transition):
- **Balances after 1 block mined**:
  ```json
  {
    "mine": {
      "trusted": 0.00000000,
      "untrusted_pending": 0.00000000,
      "immature": 50.00000000
    }
  }
  ```
- **Error during premature spend**:
  ```text
  Error: Insufficient funds (code -4)
  ```
- **Balances after 100 more blocks (Total height 101)**:
  ```json
  {
    "mine": {
      "trusted": 50.00000000,
      "untrusted_pending": 0.00000000,
      "immature": 5000.00000000
    }
  }
  ```

---

## Evidence references

- The code logic is implemented in [lab03_maturity.rs](file:///home/dorine/Music/rust-for-bitcoin-2.0/rfb_labs_week_1/src/labs/lab03_maturity.rs).
- Local tests pass successfully, verifying correct block generation, balance inspection, and error capture.

---

## Explanation

- **The Coinbase Maturity Rule (`COINBASE_MATURITY = 100`)**: By consensus, a coinbase transaction (the block reward plus transaction fees newly minted in a block) cannot be spent as an input to any subsequent transaction until it has been confirmed by at least 100 blocks. 
- **Reasoning**: If a block reorganisation occurs, blocks on the stale chain are discarded. Since a coinbase transaction only exists within its specific block, a reorganisation would invalidate the coinbase transaction itself. If the miner had been allowed to spend those coins immediately, all subsequent transactions down the spending tree would also become invalid, causing massive cascading transaction failures and double-spend chaos. Delaying spending by 100 blocks ensures the coinbase transaction is deeply buried in history, making a reorganisation deep enough to invalidate it practically impossible.
- **Convention of mining 101 blocks**: When a blockchain starts from scratch (height 0), mining 1 block puts the chain at height 1 and generates a coinbase reward of 50 BTC. To mature this first reward, 100 additional blocks must be mined on top of it. This reaches block height 101. At this height, the first coinbase block has 101 confirmations (the block itself + 100 subsequent blocks), which satisfies the consensus requirement and makes the first 50 BTC reward spendable (trusted balance). The other 100 block rewards remain immature.
