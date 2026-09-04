# Lab 03 — Coinbase maturity

## Commands used



## Terminal output

TODO: Show balances at heights 1 and 101 plus the failed premature spend.
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli -rpcwallet=miner  getbalance                                                
14949.99998350
                                                                                                                                                     
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli generatetoaddress 10000 bcrt1qxlhgvdyjxpcrexarhn76jlphkzsnpkh343kch2          
[
  "7741da750cfbea80b2e5d97c1feeda5ec85e55b3a7acb48bd4ec2f5f47c5dc50",
  "08b61697e942fb52b92278b41f147fc59dda974a531614a64676c3b4bf4c8af9",
  "3523f44a766b9299dfcf40d8404becb9f3d044726d2f4779f49b64c26e00d4a2",
  "50caa14278891637c8868a420a47d814199203b916a47cd8b92a3bb8f9893861",
  "5b86b5c7269a8b4b212404c30aa02c41fc3be9b140efc24c0cd86fac1d05a8ca",
  "237a894022d0b0c6f24b68c27725945149ecd7e8adce55d2c7e76db78aa99581",
  "491bf272b485f68e349226b9bbec93c274e3ce96ce12d7b299884ed864dac524",
  "1066da90fd34cfffdd82a958f92c10ef479989ca2697efc96816f6fb2474f0a0",
  "278198a1e2cf304eb755e6ef2183ea0fb9fb6f0dd8602ac5ef021c036353bc1e",
  "675ab009a347329184cccb286a1e1cf8188b68b1a2cac1baff575462ac18b8d1",
  "7172eab40ee783eaa1d0304cfda2c2fc6a7a5a21597018f256a16ec756d781d4",
  "4212f10a54ba6085032de70255410bca6cd1494cbb41af3c469bd1d79fccab9f",
  "2f37d58a06368c67f9d60a48eafb8f6b9685d8675e0085d00a4a0326d05bb44c",
  "10df86723bac87774de79f56e73759e154a86637f20f863ab2136afdd32d612f",
  "7688bc18c62260d0dd09435ff3044d8983b39a025132574d5696153bfe22f0b0",



## Evidence references

TODO: Link screenshots or describe the attached evidence.
![miner address](image-5.png)
![miner balance](image-6.png)
## Explanationcd ~/.bitcoin

TODO: Explain why the first coinbase reward becomes spendable at height 101.
