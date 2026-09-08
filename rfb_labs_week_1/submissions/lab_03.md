# Lab 03 — Coinbase maturity

## Commands used
These are the commands used following from lab 02
```
btc -rpcwallet=miner getnewaddress miner-address
btc generatetoaddress 1 <miner-address>
btc getblockcount
btc -rpcwallet=miner getbalances                            
btc -rpcwallet=miner sendtoaddress <Used the address from lab 01> 15  
btc -rpcwallet=miner sendtoaddress <Used the address from lab 01> 2  #Second trial with lower balance to see if it can send the untrusted balance 
btc generatetoaddress 100 <miner-address>
btc getblockcount
btc -rpcwallet=miner getbalances                            
```


## Terminal output
Here is the terminal output
```
└─$ btc -rpcwallet=miner getnewaddress miner-address
bcrt1qak6v6st6wqakcnfp7h5q9vmlkz8fs4wc8wlweh
                                                                                                           
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_1]
└─$ btc generatetoaddress 1 bcrt1qak6v6st6wqakcnfp7h5q9vmlkz8fs4wc8wlweh

[
  "54bf1e8839c05899bd9765f6304c106f002838e40fdbe3fdc86d85db50198f8f"
]
                                                                                                           
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_1]
└─$ btc getblockcount                                                   
339
                                                                                                           
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_1]
└─$ btc -rpcwallet=miner getbalances                
{
  "mine": {
    "trusted": 0.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 12.50000000
  },
  "lastprocessedblock": {
    "hash": "54bf1e8839c05899bd9765f6304c106f002838e40fdbe3fdc86d85db50198f8f",
    "height": 339
  }
}
                                                                                                           
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_1]
└─$ btc -rpcwallet=miner sendtoaddress bcrt1qtuy5ncdtd4amu4up4nw5yt9wvqtrhcsvanqz2h 15
error code: -6
error message:
Insufficient funds
                                                                                                           
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_1]
└─$ btc -rpcwallet=miner sendtoaddress bcrt1qtuy5ncdtd4amu4up4nw5yt9wvqtrhcsvanqz2h 2 
error code: -6
error message:
Insufficient funds
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_1]
└─$ btc generatetoaddress 100 bcrt1qak6v6st6wqakcnfp7h5q9vmlkz8fs4wc8wlweh           
[
  "30d8bb879281f32deeb2440699f1908bf379e2d67c1ea7c5c57d800389831ccf",
...
]
                                                                                                          
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_1]
└─$ btc getblockcount                                                     
439
                                                                                                           
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_1]
└─$ btc -rpcwallet=miner getbalances                                                 
{
  "mine": {
    "trusted": 12.50000000,
    "untrusted_pending": 0.00000000,
    "immature": 1250.00000000
  },
  "lastprocessedblock": {
    "hash": "2dc824c56d8817746a0a71e3a39101755c7696ef087d673698c915835e63858e",
    "height": 439
  }
}


```


## Evidence references
Here is the the evidence from running the projects

 ![ProjectScreenshots](evidence/Lab%203a.png)
 ![ProjectScreenshots](evidence/Lab%203b.png)


## Explanation


### Bicoin enforces a **coinbase maturity rule** the reward for mining blocks   (the "coinbase transaction") cannot be spent until it has atleast
### **100 confirmations**. The rule exists to protect against chain reorganisatain - if a node were allowed to immediately spend a freshly mined block reward,
### and that block later turned out to be not part of the winning chain (because a competting chain with more proof of work showed up ) the spent coins would never have legitimately existed at all and any transaction build on top of it would become invalid.Waiting 100 blocks makes a reorg deep enough to undo that particular block extremely unlikely in practice, so the reward is treated as safely final only once it has that much confirmation depth behind it.

### In the lab, this matches exactly what was observed: the first coinbase reward was mined at height 339 in this session (or height 1 in a freshregtest chain), and remained "immature" — reported in `getbalances` under `immature` rather than `trusted`, and rejected by `sendtoaddress` with `error code: -6, Insufficient funds` — until 100 additional blocks were mined, bringing the wallet's balance for that reward into the `trusted` category and making it spendable.
