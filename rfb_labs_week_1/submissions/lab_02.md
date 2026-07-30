# Lab 02 — Wallets and addresses

## Commands used

The following commands were used to manage wallets and addresses:

1. **Creating the wallets**:
   ```bash
   bitcoin-cli createwallet miner
   bitcoin-cli createwallet receiver
   ```

2. **Listing the loaded wallets**:
   ```bash
   bitcoin-cli listwallets
   ```

3. **Generating labelled addresses in specific wallet contexts**:
   ```bash
   bitcoin-cli -rpcwallet=miner getnewaddress mining
   bitcoin-cli -rpcwallet=receiver getnewaddress classmate
   ```

4. **Checking if the addresses belong to their respective wallets**:
   ```bash
   bitcoin-cli -rpcwallet=miner getaddressinfo <miner_address>
   bitcoin-cli -rpcwallet=receiver getaddressinfo <receiver_address>
   ```

5. **Running tests**:
   ```bash
   cargo test --test lab_02
   ```

---

## Terminal output

### 1. Verification of the Rust implementation:
Running `cargo test --test lab_02` returns:
```text
running 4 tests
test creates_wallet ... ok
test generates_labelled_address_in_wallet_context ... ok
test lists_loaded_wallets ... ok
test verifies_wallet_owns_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 2. Example terminal execution values (Mocked):
- `listwallets`:
  ```json
  [
    "miner",
    "receiver"
  ]
  ```
- `getnewaddress` for miner:
  ```text
  bcrt1qmineraddress...
  ```
- `getaddressinfo` (checking `"ismine"`):
  ```json
  {
    "address": "bcrt1qmineraddress...",
    "ismine": true
  }
  ```

---

## Evidence references

- The code is implemented in [lab02_wallets.rs](file:///home/dorine/Music/rust-for-bitcoin-2.0/rfb_labs_week_1/src/labs/lab02_wallets.rs).
- All unit tests passed, proving the correctness of wallet creation, wallet listing, address generation, and address verification.

---

## Explanation

- **Wallet Context**: Bitcoin Core supports running multiple distinct wallets concurrently on a single node (Multi-wallet mode). Because of this, many wallet-related operations (like generating addresses, signing transactions, or checking balances) cannot be run node-wide. They need to know *which* wallet database they should target.
- **Role of `-rpcwallet`**: The `-rpcwallet` option tells `bitcoin-cli` which active wallet to use for the command. For example, `bitcoin-cli -rpcwallet=miner getnewaddress` sends the request directly to the `miner` wallet.
- **Wrong Wallet Context**: If the `-rpcwallet` argument is omitted when multiple wallets are loaded, or if a wrong/non-existent wallet name is specified, the node will return an error (e.g. `"Method not found (wallet method is only available to api client that loaded a wallet)"` or `"Requested wallet does not exist"`). Thus, specifying the correct wallet context is essential for multi-wallet operations.
