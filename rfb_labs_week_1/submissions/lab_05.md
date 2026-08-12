# Lab 05 — Broadcast and mempool

## Commands used

```bash
cargo test --test lab_05
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress <receiver-address> 1
bitcoin-cli -regtest getrawmempool
bitcoin-cli -regtest -rpcwallet=miner gettransaction <payment-txid>
bitcoin-cli -regtest -rpcwallet=receiver getbalances
```

## Terminal output

The payment command returned a TXID. Before mining, `getrawmempool` contained that TXID, the sender wallet reported `confirmations=0`, and the receiver wallet showed the amount as `untrusted_pending` rather than trusted balance.

## Evidence references

Evidence is the Lab 05 test run and the mempool/payment transcript showing TXID, zero confirmations, local mempool membership, and receiver pending balance.

## Explanation

Built and signed means the transaction has valid structure and signatures. Broadcast means it was sent to the node/network. Mempool means the node accepts it as valid but unconfirmed. Confirmed means a miner included it in a valid block that became part of the active chain.
