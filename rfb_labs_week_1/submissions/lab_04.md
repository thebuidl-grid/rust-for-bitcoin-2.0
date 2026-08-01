# Lab 04 — UTXOs and outpoints

## Commands used

- List miner UTXOs: `bitcoin-cli -rpcwallet=miner listunspent`
- Miner balances: `bitcoin-cli -rpcwallet=miner getbalances`

## Terminal output

```text
selected UTXO:
  txid: e539f2ff605f56b57e3cc791a06eec6c510d7cd64a58f4f0f6f368dd70e4ef35
  vout: 0
  outpoint: e539f2ff605f56b57e3cc791a06eec6c510d7cd64a58f4f0f6f368dd70e4ef35:0
  address: bcrt1q3rvxyt9lknf9ccczql4rglx5afsrqaw0avjaud
  scriptPubKey: 001488d8622cbfb4d25c630207ea347cd4ea603075cf
  amount: 50.0 BTC
  confirmations: 101
  spendable: true

sum_spendable_utxos: 50.0 BTC
wallet_trusted_balance: 50.0 BTC
sums_match: true
```

## Evidence references
![alt text](evidence/image.png)

## Explanation

An outpoint is the unique coordinate of a transaction output: its transaction ID
plus output index (`txid:vout`). A UTXO is an output that has not yet been consumed
by a later transaction. Bitcoin does not maintain an account-style balance entry;
the wallet derives its balance by finding outputs it can spend and summing their
values. Here the single spendable 50 BTC UTXO exactly matched the trusted balance.
