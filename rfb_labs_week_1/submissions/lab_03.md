# Lab 03 — Coinbase maturity

## Commands used

TODO: Record mining, balance inspection, and premature-spend commands.
1. `cargo test --test lab_03`- for running rust test

2. ` bitcoin-cli -regtest listwallets` lists available wallets 

3. `bitcoin-cli -regtest -rpcwallet=test getnewaddress "demo"` generate a coinbase address of the miner

4.  `bitcoin-cli -regtest generatetoaddress 1 <miner_address_from_step_3_above>` — mined the first block, creating an immature coinbase reward.

5.  `bitcoin-cli -regtest -rpcwallet=test getbalances` — inspected trusted/untrusted-pending/immature balances at height 1.

6.  `bitcoin-cli -regtest -rpcwallet=test getnewaddress "receiver"` generate the miner address

7.  `bitcoin-cli -regtest -rpcwallet=<miner_wallet> sendtoaddress <receiver_address> 1` — attempted a premature spend of the immature
reward (expected to fail).
  
8. `bitcoin-cli -regtest generatetoaddress 100 <miner_address>` — mined 100 additional blocks to mature the original coinbase output.

9.`bitcoin-cli -regtest -rpcwallet=<miner_wallet> getbalances` — inspected balances again at height 101, confirming the reward became spendable.


## Terminal output

TODO: Show balances at heights 1 and 101 plus the failed premature spend.
<img width="1085" height="361" alt="Screenshot 2026-08-01 at 23 19 24" src="https://github.com/user-attachments/assets/f602df8f-f2f8-4627-9ed4-7fd20b059b61" />

<img width="1087" height="265" alt="Screenshot 2026-08-01 at 23 23 54" src="https://github.com/user-attachments/assets/ba0ef11e-a2f8-4c64-bb72-ac616904a479" />

<img width="1100" height="379" alt="Screenshot 2026-08-01 at 23 27 40" src="https://github.com/user-attachments/assets/60ed846d-8fa8-4058-a154-a7cfd5e48769" />

## Evidence references

TODO: Link screenshots or describe the attached evidence.

First wallet was created, i.e., `test`, then the address followed. The first block was mined, at which point the trusted party was still 0.00, and the height was just 1, given that we only have one mined block, hence blockcount is 1.

We attempted to send 1 BTC to someone(at this point, we had to generate a receiver's address), but this resulted in a failed transaction.

After an additional 100 blocks were mined, the same failed transaction went through, and in the process, we paid a network fee, which resulted in the decrement of the `trusted` from 100.00 to 99.999

## Explanation

TODO: Explain why the first coinbase reward becomes spendable at height 101.

Bitcoin Core enforces a **coinbase maturity rule**: a block's coinbase reward (the newly minted subsidy plus fees) cannot be spent until it has **100 confirmations**. This is a consensus rule, not a wallet preference — a transaction spending an immature coinbase output would be rejected by the network.

At height 1 (immediately after mining the first block), the reward exists but sits in the `immature` balance bucket — the wallet can see it, but `sendtoaddress` fails with "Insufficient funds" because none of the spendable (`trusted`) balance covers it.

Mining 100 more blocks advances the chain to height 101(102 in my case because of the additional block  I mined), at which point the original block (height 1) has exactly more than 100 confirmations behind it. At that point, Bitcoin Core reclassifies that coinbase output from `immature` to `trusted`, making it spendable. This is why the reward "unlocks" specifically at height 101, not 100 
