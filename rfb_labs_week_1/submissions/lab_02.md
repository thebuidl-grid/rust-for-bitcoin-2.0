# Lab 02 — Wallets and addresses

## Commands used

- Create miner wallet: `bitcoin-cli createwallet miner`
- Create receiver wallet: `bitcoin-cli createwallet receiver`
- List loaded wallets: `bitcoin-cli listwallets`
- Miner address: `bitcoin-cli -rpcwallet=miner getnewaddress mining`
- Receiver address: `bitcoin-cli -rpcwallet=receiver getnewaddress classmate`
- Verify miner ownership: `bitcoin-cli -rpcwallet=miner getaddressinfo "$MINER_ADDRESS"`
- Verify receiver ownership: `bitcoin-cli -rpcwallet=receiver getaddressinfo "$RECEIVER_ADDRESS"`

## Terminal output

```text
loaded_wallets: ["", "miner", "receiver"]

miner:
  label: mining
  address: bcrt1q3rvxyt9lknf9ccczql4rglx5afsrqaw0avjaud
  starts_with_bcrt1: true
  is_mine: true

receiver:
  label: classmate
  address: bcrt1qn0a5sawrhah5wfdacckskhsxvmf068r34ktv3d
  starts_with_bcrt1: true
  is_mine: true
```

## Evidence references
![alt text](evidence/image-7.png)

## Explanation

Bitcoin Core can load several wallets at once, so wallet-specific RPC calls need a
wallet context. `-rpcwallet=miner` directs a call to the miner wallet, while
`-rpcwallet=receiver` directs it to the receiver wallet. Using the wrong context
means Bitcoin Core searches or acts through the wrong wallet: an address may report
`ismine: false`, or a payment may use the wrong wallet's coins.
