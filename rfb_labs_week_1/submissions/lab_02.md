# Lab 02 — Wallets and addresses

## Commands used

docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 getwalletinfo   

docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 getnewaddress   


## Terminal output


{
  "walletname": "mywallet1",
  "walletversion": 169900,
  "format": "sqlite",
  "balance": 50.00000000,
  "unconfirmed_balance": 0.00000000,
  "immature_balance": 5000.00000000,
  "txcount": 101,
  "keypoolsize": 4000,
  "keypoolsize_hd_internal": 4000,
  "paytxfee": 0.00000000,
  "private_keys_enabled": true,
  "avoid_reuse": false,
  "scanning": false,
  "descriptors": true,
  "external_signer": false,
  "blank": false,
  "birthtime": 1785752735,
  "lastprocessedblock": {
    "hash": "7368ad4272240cf3f9bd35ea40ab623b6abdb52765a862170951e5e735a84180",
    "height": 101
  }

## Evidence references


https://drive.google.com/drive/folders/1HvmkTC2bazkXgBELjgbLaaW8grJQgF9h?usp=sharing

## Explanation


A single running Bitcoin Core node can have multiple wallets loaded simultaneously.But many RPC methods are inherently wallet-scoped — they need to know which wallet's keys/UTXOs/history to operate on as the method name alone doesn't specify that so rpcwallet<name> is how we supply that missing piece.

