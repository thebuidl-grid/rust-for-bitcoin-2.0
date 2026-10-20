# Lab 07 — Confirmation and block membership

## Commands used

TODO: Record the mining, mempool, transaction, and block commands.

1. `cargo test --test lab_07`
2. `bitcoin-cli -regtest -rpcwallet=test sendtoaddress  <receiver_addr> 1` - receiver address was created earlier in the lab.
3. `bitcoin-cli -regtest getrawmempool`
4. ` bitcoin-cli -regtest generatetoaddress 1 <addr_on_the test_wallet>`.
5. `bitcoin-cli -regtest getrawmempool`
6. `bitcoin-cli -regtest -rpcwallet=receiver gettransaction <txid_of_the_recerver_above>`
7. `bitcoin-cli -regtest getblock <block_hash_of_the> 1`

## Terminal output

TODO: Show the empty mempool, confirmation count, block hash, and TXID in block.
<img width="1132" height="207" alt="Screenshot 2026-08-02 at 13 48 05" src="https://github.com/user-attachments/assets/b0c52bee-ef05-49e2-b1f9-989af59e1634" />
<img width="1138" height="570" alt="Screenshot 2026-08-02 at 13 49 12" src="https://github.com/user-attachments/assets/9f17e79c-c9ab-4a44-a90a-0776673bdd1b" />
<img width="1131" height="475" alt="Screenshot 2026-08-02 at 13 49 43" src="https://github.com/user-attachments/assets/cd31d5eb-5548-4b49-94ec-caf2ca16a390" />

## Evidence references

TODO: Link screenshots or describe the attached evidence.

first we send I BTC from the test wallet to the receiver_addr, we get the list of transactions in the mempool using `getrawmempool`, and proceed to mine a block to confirm the transaction to the address from the `test` wallet. At this stage, running `getrawmempool` returns empty, confirming the block has been mined(the transaction moved from mempool into the block). Get the transaction details(with blockhash obtained here) we run getblock to confirm everything sinks.

## Explanation

TODO: Explain exactly what changed when the transaction became confirmed.

Mining one block moved the transaction from the node’s mempool into a confirmed block. The mempool became empty, the transaction gained 1 confirmation, and Bitcoin Core recorded the block hash containing it. Verifying that the block’s tx array contains the TXID proves the transaction is now part of the blockchain rather than only a pending mempool transaction.
