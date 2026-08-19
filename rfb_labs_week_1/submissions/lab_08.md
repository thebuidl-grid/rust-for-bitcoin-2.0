# Lab 08 — Block security

## Commands used

TODO: Record block-header inspection and additional mining commands.

1. `cargo test --test lab_08`
2. `bitcoin-cli -regtest getblockheader  <block_hash>`- `block_hash` gotten from `lab_07.md`
3. `bitcoin-cli -regtest -rpcwallet=receiver gettransaction <txid>` - `txid` of the receiver
4. `bitcoin-cli -regtest generatetoaddress 5 <demo_address_created_earlier>` - the `demo_addr` is like the `miner's addr` in this instance
5. `bitcoin-cli -regtest -rpcwallet=receiver gettransaction <txid>` 


## Terminal output

TODO: Show header fields and confirmation count changing from one to six.
<img width="1135" height="278" alt="Screenshot 2026-08-02 at 14 26 54" src="https://github.com/user-attachments/assets/c717120b-0c6e-45f3-9e16-c55ab2c30506" />
<img width="1141" height="572" alt="Screenshot 2026-08-02 at 14 27 56" src="https://github.com/user-attachments/assets/2286b93e-45e1-4391-8aea-2c426cbbf18a" />
<img width="1131" height="678" alt="Screenshot 2026-08-02 at 14 28 35" src="https://github.com/user-attachments/assets/b05ff4d7-8016-4875-af48-a7560f60eb68" />



## Evidence references

TODO: Link screenshots or describe the attached evidence.

First, we obtained the blockheader using the `getblockheader` and the `block_hash`, then we took note of the number of confirmations(1 in this case) using `bitcoin-cli -regtest -rpcwallet=receiver gettransaction <txid>`. After this stage, we mined additonal 5 blocks to the miner's address, thus we can see the confirmations move from 1 to 6

## Explanation

TODO: Explain hash links, Merkle roots, proof of work, and confirmation depth.

The block header contains the previous block hash, which links blocks together. The Merkle root commits to all transactions in the block. The nonce, bits, difficulty, and chainwork provide proof-of-work evidence. Mining five additional blocks increased the transaction’s confirmation depth from 1 to 6, meaning six blocks now secure the transaction and make reversal significantly harder.
