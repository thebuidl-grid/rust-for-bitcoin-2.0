# Lab 02 — Wallets and addresses

## Commands used
### bitcoin-cli
bitcoin-cli -regtest createwallet "miner"
bitcoin-cli -regtest createwallet "receiver"
bitcoin-cli -regtest listwallets
bitcoin-cli -regtest -rpcwallet=miner getnewaddress "mining"
bitcoin-cli -regtest -rpcwallet=receiver getnewaddress "classmate"
bitcoin-cli -regtest -rpcwallet=miner getaddressinfo <miner-address>
bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo <receiver-address>
### for the rust code 
cargo test --test lab_02 -- --nocapture

## Terminal output
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli createwallet miner     
{
  "name": "miner"
}
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli createwallet reciver 
{
  "name": "reciver"
}
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli listwallet           
error code: -32601
error message:
Method not found
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli listwallets
[
  "testwallet",
  "miner",
  "reciver"
]
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli -rpcwallet=miner getnewaddress"mining" 
error code: -32601
error message:
Method not found
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli -rpcwallet=miner getnewaddress "mining"
bcrt1qxlhgvdyjxpcrexarhn76jlphkzsnpkh343kch2
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli -rpcwallet=reciver getnewaddress "classmate"
bcrt1q46zu9fas4cpr9uxjenfek30q77mjfj78p0333w
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli -rpcwallet=miner getaddressinfo <miner.address>
zsh: parse error near `\n'
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli -rpcwallet=miner getaddressinfo <miner-address>
zsh: parse error near `\n'
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ 
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli -rpcwallet=miner getaddressinfo mining         
error code: -5
error message:
Invalid checksum or length of Base58 address (P2PKH or P2SH)
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli -rpcwallet=miner getaddressinfo <miner-address>
zsh: parse error near `\n'
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli -regtest -rpcwallet=miner getaddressinfo <miner-address>
bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo <receiver-address>
zsh: parse error near `\n'
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli -regtest -rpcwallet=miner getaddressinfo <miner-address> &&
bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo <receiver-address>
zsh: parse error near `&&'
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli -regtest -rpcwallet=miner getaddressinfo bcrt1qxlhgvdyjxpcrexarhn76jlphkzsnpkh343kch2 &&
bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo bcrt1q46zu9fas4cpr9uxjenfek30q77mjfj78p0333w

{
  "address": "bcrt1qxlhgvdyjxpcrexarhn76jlphkzsnpkh343kch2",
  "scriptPubKey": "001437ee86349230703c9ba3bcfda97c37b0a130daf1",
  "ismine": true,
  "solvable": true,
  "desc": "wpkh([cf4090ca/84h/1h/0h/0/0]03235fa87afad4647aeddcb6d8140d82b03f5d61149cd0cbab38303dbe56537271)#yfdmvdq5",
  "parent_desc": "wpkh([cf4090ca/84h/1h/0h]tpubDCJASerPBw7XeLC93JeswqAiLdeQ1G3omhKU3Uh87uF2JzHTVFjCGsUS9oX3wxNv5XGT81SfyqoPRsFqe8zVbFCGUmSEtcWQ9rZEgzH4C34/0/*)#9r6s0umf",
  "iswatchonly": false,
  "isscript": false,
  "iswitness": true,
  "witness_version": 0,
  "witness_program": "37ee86349230703c9ba3bcfda97c37b0a130daf1",
  "pubkey": "03235fa87afad4647aeddcb6d8140d82b03f5d61149cd0cbab38303dbe56537271",
  "ischange": false,
  "timestamp": 1785754104,
  "hdkeypath": "m/84h/1h/0h/0/0",
  "hdseedid": "0000000000000000000000000000000000000000",
  "hdmasterfingerprint": "cf4090ca",
  "labels": [
    "mining"
  ]
}
error code: -18
error message:
Requested wallet does not exist or is not loaded
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ 
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo bcrt1q46zu9fas4cpr9uxjenfek30q77mjfj78p0333w
error code: -18
error message:
Requested wallet does not exist or is not loaded
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo bcrt1q46zu9fas4cpr9uxjenfek30q77mjfj78p0333 
error code: -18
error message:
Requested wallet does not exist or is not loaded
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli -regtest -rpcwallet=miner getaddressinfo bcrt1qxlhgvdyjxpcrexarhn76jlphkzsnpkh343kch2    

{
  "address": "bcrt1qxlhgvdyjxpcrexarhn76jlphkzsnpkh343kch2",
  "scriptPubKey": "001437ee86349230703c9ba3bcfda97c37b0a130daf1",
  "ismine": true,
  "solvable": true,
  "desc": "wpkh([cf4090ca/84h/1h/0h/0/0]03235fa87afad4647aeddcb6d8140d82b03f5d61149cd0cbab38303dbe56537271)#yfdmvdq5",
  "parent_desc": "wpkh([cf4090ca/84h/1h/0h]tpubDCJASerPBw7XeLC93JeswqAiLdeQ1G3omhKU3Uh87uF2JzHTVFjCGsUS9oX3wxNv5XGT81SfyqoPRsFqe8zVbFCGUmSEtcWQ9rZEgzH4C34/0/*)#9r6s0umf",
  "iswatchonly": false,
  "isscript": false,
  "iswitness": true,
  "witness_version": 0,
  "witness_program": "37ee86349230703c9ba3bcfda97c37b0a130daf1",
  "pubkey": "03235fa87afad4647aeddcb6d8140d82b03f5d61149cd0cbab38303dbe56537271",
  "ischange": false,
  "timestamp": 1785754104,
  "hdkeypath": "m/84h/1h/0h/0/0",
  "hdseedid": "0000000000000000000000000000000000000000",
  "hdmasterfingerprint": "cf4090ca",
  "labels": [
    "mining"
  ]
}
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ bitcoin-cli -rpcwallet=reciver getaddressinfo bcrt1q46zu9fas4cpr9uxjenfek30q77mjfj78p0333w        
{
  "address": "bcrt1q46zu9fas4cpr9uxjenfek30q77mjfj78p0333w",
  "scriptPubKey": "0014ae85c2a7b0ae0232f0d2ccd39b45e0f7b724cbc7",
  "ismine": true,
  "solvable": true,
  "desc": "wpkh([0b8555d8/84h/1h/0h/0/0]0224d5fe67f50410d8b009902ef96134a4b30681b5343584767aa606b86a4ea47e)#cg4vvnrn",
  "parent_desc": "wpkh([0b8555d8/84h/1h/0h]tpubDDpn2WeKSRQXeMvhWvAKjphe7CfcK5DAFi2cENhmgiJ413dwLPtvSKEtbLdnQhdPRVJmtcYVPwxC8hebaEX9H3ZpgdeQtgzBNnqVVUE6Fjy/0/*)#vcdmmw9h",
  "iswatchonly": false,
  "isscript": false,
  "iswitness": true,
  "witness_version": 0,
  "witness_program": "ae85c2a7b0ae0232f0d2ccd39b45e0f7b724cbc7",
  "pubkey": "0224d5fe67f50410d8b009902ef96134a4b30681b5343584767aa606b86a4ea47e",
  "ischange": false,
  "timestamp": 1785754115,
  "hdkeypath": "m/84h/1h/0h/0/0",
  "hdseedid": "0000000000000000000000000000000000000000",
  "hdmasterfingerprint": "0b8555d8",
  "labels": [
    "classmate"
  ]
}
                                                                                                                                                                                              
┌──(kellymusk㉿GHOSTMUSK)-[~]
└─$ 



## Evidence references


![alt text](image-1.png)
![alt text](image-2.png)
![alt text](image-3.png)
![alt text](image-4.png)

## Explanation

Wallet context means “which wallet are we asking?” Bitcoin Core can host multiple wallets at once, but each wallet has its own keys and addresses. -rpcwallet=miner forces the command to run in the miner wallet, while -rpcwallet=receiver runs it in the receiver wallet. If you request getaddressinfo in the wrong wallet, the address may not be marked as yours even though it belongs to a different loaded wallet.
-rpcwallet tells Bitcoin Core which wallet to use for a wallet-scoped RPC call
