# Lab 09 — Multi-UTXO coin selection

## Commands used

- Create Alice wallet: `bitcoin-cli createwallet alice`
- Alice address: `bitcoin-cli -rpcwallet=alice getnewaddress alice-funding`
- First funding payment: `bitcoin-cli -rpcwallet=miner sendtoaddress "$ALICE_ADDRESS" 0.4`
- Second funding payment: `bitcoin-cli -rpcwallet=miner sendtoaddress "$ALICE_ADDRESS" 0.4`
- Third funding payment: `bitcoin-cli -rpcwallet=miner sendtoaddress "$ALICE_ADDRESS" 0.4`
- Confirm funding: `bitcoin-cli generatetoaddress 1 "$MINER_ADDRESS"`
- Alice UTXOs: `bitcoin-cli -rpcwallet=alice listunspent`
- Receiver address: `bitcoin-cli -rpcwallet=receiver getnewaddress lab09-receiver`
- Combined payment: `bitcoin-cli -rpcwallet=alice sendtoaddress "$RECEIVER_ADDRESS" 1`
- Decode combined spend: `bitcoin-cli getrawtransaction "$SPEND_TXID" 2`

## Terminal output

```text
Alice address: bcrt1qf8jz6k7aezxgw9m3y4nu2emp4s95ygwc3ykxqy

confirmed 0.4 BTC funding outpoints:
  457b9a4603b4a226561f2fbb800692d339db76b35b2b29725d8a25c3675d1656:1
  1b587291c7fe67a2d9e635c9fba869a9be5f46e706a0fe9e8ed235b1f103c914:1
  5873fc14add9081bd81f915934fd966f85fefc6642ee56f27449a290bf69b491:0

Each UTXO:
  amount: 0.4 BTC
  confirmations: 1
  spendable: true
  scriptPubKey: 001449e42d5bddc88c8717712567c56761ac0b4221d8

combined spend:
  txid: 622dc4367d6563b1dbdff9c5b6a1ff18685aac69505bdfd69dc2bcdfcb086312
  input count: 3
  receiver output: 1.00000000 BTC
  receiver address: bcrt1qnkyad48qjv6pccwdn58f2d603dejhm4m86tlea
  change output: 0.19994480 BTC
  change address: bcrt1q44wlqqyjpl7k06q39nxp98rg5hhrynzufpw07y
  fee: 0.00005520 BTC

1.20000000 = 1.00000000 + 0.19994480 + 0.00005520 BTC
```

## Evidence references
![alt text](image-9.png)
## Explanation

Each selected input spends its referenced UTXO completely; inputs are not partially
consumed. Alice needed all three 0.4 BTC UTXOs to fund the 1 BTC payment. The wallet
returned the surplus to Alice as a 0.19994480 BTC change output, with 0.00005520 BTC
left as the fee. Using several inputs together provides evidence that one entity
controls all their signing keys, so coin consolidation can weaken privacy even when
the addresses differ.
