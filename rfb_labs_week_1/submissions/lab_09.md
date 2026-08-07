# Lab 09 — Multi-UTXO coin selection

## Commands used

TODO: Record funding, confirmation, spending, and decoding commands.

1. `cargo test --test lab_09`
2. `bitcoin-cli -regtest createwallet "alice"`
3. `bitcoin-cli -regtest -rpcwallet=alice getnewaddress "alice_address"`
4. `bitcoin-cli -regtest -rpcwallet=test sendtoaddress bcrt1qlqy0umqufttcsyuxx8uvrud9sral7xu9urjrt8 0.4` * 3
5. `bitcoin-cli -regtest -rpcwallet=alice listunspent` - empty at this stage
6. `bitcoin-cli -regtest -rpcwallet=alice getbalances` - check balance to confirm untrusted_pending is  `"untrusted_pending": 1.20000000,` as it should.
7. `bitcoin-cli -regtest -rpcwallet=test getnewaddress`
8. `bitcoin-cli -regtest generatetoaddress 1 <addr_from_7>`
9. `bitcoin-cli -regtest -rpcwallet=alice listunspent` - no longer empty
10. ` bitcoin-cli -regtest -rpcwallet=alice getbalances` - balance upgrades from  `"untrusted_pending": 1.20000000,` to  `"trusted": 1.20000000`
11. ` bitcoin-cli -regtest -rpcwallet=alice sendtoaddress<reciever_addr> 1` spending phase
12. `bitcoin-cli -regtest getrawtransaction <spend_tx_from_11_above`


## Terminal output

TODO: Show Alice's three UTXOs and the combined transaction inputs and outputs.
<img width="817" height="64" alt="Screenshot 2026-08-02 at 17 15 30" src="https://github.com/user-attachments/assets/ae11f9b0-dcf1-49f4-9d29-1c5e13eba53b" />
<img width="1128" height="369" alt="Screenshot 2026-08-02 at 17 17 22" src="https://github.com/user-attachments/assets/b25cf07b-33f8-4919-a150-a1a6580635fc" />
<img width="1125" height="91" alt="Screenshot 2026-08-02 at 17 29 52" src="https://github.com/user-attachments/assets/947bc085-1b42-4a97-b564-0b0f0ef68d83" />
<img width="1129" height="704" alt="Screenshot 2026-08-02 at 17 31 17" src="https://github.com/user-attachments/assets/d915e446-62ca-4174-b9cb-c9649f8530cf" />
<img width="1133" height="49" alt="Screenshot 2026-08-02 at 17 41 38" src="https://github.com/user-attachments/assets/2be34a34-ff17-4c38-b37b-9018e47ac567" />
<img width="1135" height="721" alt="Screenshot 2026-08-02 at 17 42 51" src="https://github.com/user-attachments/assets/159f0a40-638e-47cd-96b7-8cfc6db58e54" />
<img width="1127" height="453" alt="Screenshot 2026-08-02 at 17 43 19" src="https://github.com/user-attachments/assets/70ae4ac4-a7cf-4bee-ad00-3a4c501eba61" />




## Evidence references

TODO: Link screenshots or describe the attached evidence.

We created a wallet for Alice, with an address to go with the wallet, then sent o.4 BTC 3 times from the miner's wallet(test wallet) to Alice's address. currently alice has not spent anything, hence her balance is still at untrusted_pending with the value of `1.2000(0.4*3)
`. At this stage alice listunspent is no longer empty, and Alice's balance has moved from `untrusted_pending` to `trusted`. With balance set as trusted, Alice can now send funds to someone else, and we successfully retrieved the `getrawtransaction`.

## Explanation

TODO: Explain input combination, change, fees, and the privacy implication.

Alice did not have a single 1 BTC UTXO, so Bitcoin Core combined three smaller UTXOs to fund the payment. The receiver received 1 BTC, while the remaining value was returned to Alice as a change output. The small difference between total inputs and total outputs is the miner fee. Combining multiple inputs links those UTXOs together on-chain, which can reduce privacy by revealing that they are likely controlled by the same wallet.
