# Lab 02 — Wallets and addresses

## Commands used

```bash
cargo test --test lab_02
```

RPC methods called:
- `createwallet <name>` - Create a new wallet
- `listwallets` - List all loaded wallets
- `getnewaddress <label>` - Generate a new address with a label in wallet context
- `getaddressinfo <address>` - Check if wallet owns an address

## Terminal output

```
running 4 tests
test creates_wallet ... ok
test generates_labelled_address_in_wallet_context ... ok
test lists_loaded_wallets ... ok
test verifies_wallet_owns_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All tests pass successfully, confirming:
- Wallets can be created with `createwallet`
- Multiple wallets can be listed with `listwallets`
- Addresses can be generated in wallet context with labels
- Wallet ownership of addresses can be verified via `ismine` field

## Explanation

Lab 02 demonstrates Bitcoin wallet management and address generation:

1. **Wallet Context**: Bitcoin Core supports multiple wallets, managed via the `-rpcwallet` parameter. All RPC calls can specify which wallet to use, allowing parallel management of independent wallet states.

2. **Address Generation**: Each wallet can generate receiving addresses. Labels help organize and identify addresses by purpose (e.g., "income", "expenses"). The `getnewaddress` RPC generates new addresses deterministically from the wallet's seed.

3. **Address Ownership**: The `getaddressinfo` RPC returns the `ismine` field, indicating whether the wallet controls the private key for an address. This is essential for proving wallet ownership before spending.

4. **Key Insight**: Wallets are containers for private keys. Creating a wallet doesn't create addresses - addresses are generated on demand and tracked within the wallet. This lab demonstrates the basic lifecycle of wallet and address creation.
