# Lab 07 — Confirmation and block membership

## Commands used

```bash
cargo test --test lab_07
bitcoin-cli -regtest generatetoaddress 1 <miner-address>
bitcoin-cli -regtest getrawmempool
bitcoin-cli -regtest -rpcwallet=receiver gettransaction <payment-txid>
bitcoin-cli -regtest getblock <confirming-block-hash> 1
```

## Terminal output

After mining one block, the mempool was empty. The receiver wallet reported the payment with `confirmations=1` and a `blockhash`. The verbose block transaction list contained the payment TXID.

## Evidence references

Evidence is the Lab 07 test run and the confirmation transcript showing empty mempool, receiver transaction status, confirming block hash, and TXID membership in the block.

## Explanation

Mining did not change the serialized transaction. It changed the transaction's status by committing it into a block in the active chain. The transaction moved from a local mempool candidate into agreed block history with one confirmation.
