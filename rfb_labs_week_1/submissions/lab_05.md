# Lab 05 — Broadcast and mempool

## Commands used

TODO: Record the payment, mempool, transaction, and balance commands.

1. `cargo test --test lab_05`
2. `bitcoin-cli -regtest createwallet "receiver"` - create a new wallet for the receiver
3. `bitcoin-cli -regtest -rpcwallet=test getnewaddress "receiver_address"` - create an address for the receiver
4. `bitcoin-cli -regtest -rpcwallet=test sendtoaddress <receiver_address> 1` - send 1 BTC to the newly created address in 3 above
5. `bitcoin-cli -regtest getrawmempool`
6. `bitcoin-cli -regtest -rpcwallet=test gettransaction <txid_from_number_5_above>`.
7. `bitcoin-cli -regtest -rpcwallet=receiver getbalances`



## Terminal output

TODO: Show the TXID, zero confirmations, mempool entry, and pending balance.
<img width="848" height="60" alt="Screenshot 2026-08-02 at 02 32 58" src="https://github.com/user-attachments/assets/a2c84795-74b1-44f4-8fc8-305cda1dfcea" />
<img width="1091" height="159" alt="Screenshot 2026-08-02 at 02 33 55" src="https://github.com/user-attachments/assets/7bd3b43c-b834-49db-aa17-de7996a428c0" />
<img width="1118" height="715" alt="Screenshot 2026-08-02 at 02 34 57" src="https://github.com/user-attachments/assets/d664b347-c8a6-408d-ae13-06bfa6b733c2" />



## Evidence references

TODO: Link screenshots or describe the attached evidence.

The receiver's wallet successfully received 1 BTC, but the transaction is still unconfirmed (confirmations = 0, trusted = false). The amount appears in untrusted_pending = 1.00000000, confirming it is in the mempool and not yet spendable.



## Explanation

TODO: Distinguish signed, broadcast, mempool, and confirmed states.

Signed means the wallet created and signed the transaction locally.
Broadcast: the transaction was sent to the Bitcoin Core node.
Mempool: the node accepted the transaction and stored it in getrawmempool.
Confirmed: after mining a block, confirmations become > 0, and the receiver's balance moves from untrusted_pending to trusted.
