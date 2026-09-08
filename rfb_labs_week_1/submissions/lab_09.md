# Lab 09 — Multi-UTXO coin selection

## Commands used

I used a couple of commands from creating alice wallet to sending btc to decoding utxo:
```
# 1. Creating Alice's wallet and geting her receiving address
btc createwallet alice
btc -rpcwallet=alice getnewaddress alice-address


# 2. Sending three separate 0.4 BTC funding transactions from miner to Alice
btc -rpcwallet=miner sendtoaddress <alice-address> 0.4
btc -rpcwallet=miner sendtoaddress <alice-address> 0.4
btc -rpcwallet=miner sendtoaddress <alice-address> 0.4

# 3. Mining a block to confirm all three funding transactions
btc generatetoaddress 1 bcrt1qak6v6st6wqakcnfp7h5q9vmlkz8fs4wc8wlweh

# 4. Confirming Alice now holds three separate confirmed UTXOs
btc -rpcwallet=alice listunspent


# 5. Geting a receiver address (reusing the earlier "receiver" wallet)
btc -rpcwallet=receiver getnewaddress payout


# 6. Sending 1 BTC from Alice — this requires combining more than one of her 0.4 BTC UTXOs
btc -rpcwallet=alice sendtoaddress <receiver-address> 1


# 7. Decoding the spend and prove multiple inputs were used
btc getrawtransaction <spend-txid> 2

# 8.  Alice's remaining UTXO set afterward
btc -rpcwallet=alice listunspent
# should show the change output only (if any), her three funding UTXOs now spent
```

## Terminal output

The Terminal output was quite long showed the proof for step 7 in screenshot below

## Evidence references

![ProjectScreenshot](evidence/Lab%209.png)

## Explanation

Since none of Alice's individual UTXOs (0.4 BTC each) covered the 1 BTC
payment alone, her wallet combined three of them as inputs to meet the
total. The leftover after payment and fee (0.4×3 − 1 − fee) was returned
to Alice as a change output back to her own wallet. Fees are simply the
gap between total input value and total output value, paid implicitly to
the miner rather than sent to any address. The privacy implication is
that combining multiple UTXOs in one transaction publicly links them
together on-chain — anyone observing the blockchain can infer that all
three inputs likely belong to the same person, even if the outputs
individually don't reveal ownership.
