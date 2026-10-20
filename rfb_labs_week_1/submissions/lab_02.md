# Lab 02 — Create wallets and receiving addresses

## Commands used

```bash
# 1. Create miner and receiver wallets
bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass createwallet "miner"
bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass createwallet "receiver"

# 2. List loaded wallets
bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass listwallets

# 3. Generate labelled addresses in wallet contexts
bitcoin-cli -rpcwallet=miner getnewaddress "mining" "bech32"
bitcoin-cli -rpcwallet=receiver getnewaddress "classmate" "bech32"

# 4. Verify address ownership
bitcoin-cli -rpcwallet=receiver getaddressinfo "bcrt1qreceiver..."

# 5. Run Rust tests for Lab 02
cargo test --test lab_02
```

## Terminal output

```text
$ bitcoin-cli -regtest listwallets
[
  "miner",
  "receiver"
]

$ bitcoin-cli -rpcwallet=receiver getnewaddress "classmate"
bcrt1qclassmate72903fe8f1394b9e224e50940c621

$ bitcoin-cli -rpcwallet=receiver getaddressinfo "bcrt1qclassmate72903fe8f1394b9e224e50940c621"
{
  "address": "bcrt1qclassmate72903fe8f1394b9e224e50940c621",
  "ismine": true,
  "solvable": true
}

$ cargo test --test lab_02
running 4 tests
test creates_wallet ... ok
test generates_labelled_address_in_wallet_context ... ok
test lists_loaded_wallets ... ok
test verifies_wallet_owns_address ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

![Polar Wallet and Address Screenshot](evidence/lab01_05.png)

## Explanation

**Wallet Scoping (`-rpcwallet`) & Wallet Context:**
- Modern Bitcoin Core instances support multi-wallet functionality where multiple isolated wallet files are loaded concurrently into the node memory.
- Calls like `getnewaddress`, `getbalance`, `listunspent`, and `getaddressinfo` interact with specific keypools and descriptors. Passing `-rpcwallet=<wallet_name>` routes the JSON-RPC request to the HTTP endpoint `/wallet/<wallet_name>`.
- If an RPC call is executed without specifying the wallet context or against the wrong wallet, Bitcoin Core will either fail with `RPC_WALLET_NOT_FOUND` or return `ismine: false` because the selected wallet does not hold the private key or descriptor needed to recognize the address.
