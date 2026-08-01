# Lab 02 — Wallets and addresses

## Commands used

<!-- TODO: Record how you created and inspected both wallets and addresses. -->
```bash
# Create miner wallet
bitcoin-cli createwallet "miner" 

# Create receiver wallet
bitcoin-cli createwallet "receiver" 

# List all wallets
bitcoin-cli listwallets   

# Generate address in miner wallet (wallet-scoped, needs -rpcwallet)
bitcoin-cli -rpcwallet=miner getnewaddress "miner1"

# Generate address in receiver wallet (wallet-scoped, needs -rpcwallet)
bitcoin-cli -rpcwallet=receiver getnewaddress "classmate"

# Check address ownership (wallet-scoped, needs -rpcwallet)
bitcoin-cli  -rpcwallet=miner getaddressinfo "bcrt1q..."

# Check address ownership (wallet-scoped, needs -rpcwallet)
bitcoin-cli  -rpcwallet=receiver getaddressinfo "bcrt1q..."
```

## Terminal output

<!-- TODO: Include loaded wallets, addresses, and ownership evidence. -->
```bash
bitcoin@backend1:/$ bitcoin-cli createwallet "miner"
{
  "name": "miner"
}
bitcoin@backend1:/$ bitcoin-cli createwallet "receiver"
{
  "name": "receiver"
}
bitcoin@backend1:/$ bitcoin-cli listwallets createwallet           
error code: -1
error message:
listwallets

Returns a list of currently loaded wallets.
For full information on the wallet, use "getwalletinfo"

Result:
[           (json array)
  "str",    (string) the wallet name
  ...
]

Examples:
> bitcoin-cli listwallets 
> curl --user myusername --data-binary '{"jsonrpc": "2.0", "id": "curltest", "method": "listwallets", "params": []}' -H 'content-type: application/json' http://127.0.0.1:8332/

bitcoin@backend1:/$ bitcoin-cli listwallets              
[
  "",
  "miner",
  "receiver"
]
bitcoin@backend1:/$ bitcoin-cli -rpcwallet="miner" getnewaddress "miner1"
bcrt1qn893ldl3w0zt5myjm0lxh3kpreedtwtnsc0272
bitcoin@backend1:/$ bitcoin-cli -rpcwallet="receiver" getnewaddress "classmate"
bcrt1qcs67lj6q6up4nadjggckcpek27wtrkgz8h58wr
bitcoin@backend1:/$ bitcoin-cli -rpcwallet="miner" getaddressinfo "bcrt1qn893ldl3w0zt5myjm0lxh3kpreedtwtnsc0272"
{
  "address": "bcrt1qn893ldl3w0zt5myjm0lxh3kpreedtwtnsc0272",
  "scriptPubKey": "001499cb1fb7f173c4ba6c92dbfe6bc6c11e72d5b973",
  "ismine": true,
  "solvable": true,
  "desc": "wpkh([bf85dada/84h/1h/0h/0/0]02c1f38a06d513232a1163d22d0814828209ac7d0e61f569feddf87d5741c7f940)#rxg2w00t",
  "parent_desc": "wpkh([bf85dada/84h/1h/0h]tpubDCGB4LcrzswU2LJkEqxbFexmP726AfBxavYnPJneZPGDdQY1tL3Xhi3mnsEmKpmBDGC38kbNL36NjdetaFaSLtsDk9HAtSkn4xV6oyt2AVm/0/*)#5le9jjll",
  "iswatchonly": false,
  "isscript": false,
  "iswitness": true,
  "witness_version": 0,
  "witness_program": "99cb1fb7f173c4ba6c92dbfe6bc6c11e72d5b973",
  "pubkey": "02c1f38a06d513232a1163d22d0814828209ac7d0e61f569feddf87d5741c7f940",
  "ischange": false,
  "timestamp": 1785578664,
  "hdkeypath": "m/84h/1h/0h/0/0",
  "hdseedid": "0000000000000000000000000000000000000000",
  "hdmasterfingerprint": "bf85dada",
  "labels": [
    "miner1"
  ]
}
bitcoin@backend1:/$ bitcoin-cli -rpcwallet="receiver" getaddressinfo "bcrt1qcs67lj6q6up4nadjggckcpek27wtrkgz8h58wr"
{
  "address": "bcrt1qcs67lj6q6up4nadjggckcpek27wtrkgz8h58wr",
  "scriptPubKey": "0014c435efcb40d70359f5b242316c0736579cb1d902",
  "ismine": true,
  "solvable": true,
  "desc": "wpkh([53a25ce0/84h/1h/0h/0/0]0282b5b54f1c99e6d385f13517bb08a81cfc1ef1737596c503511fcb8cb2b499f0)#0g3uudlv",
  "parent_desc": "wpkh([53a25ce0/84h/1h/0h]tpubDDCzefcTRKbdpPYBDnj8So36ZQx7DVcTYWKYsoGv1NwaPNUkcQVQVHLL4UX3coBWG93oLiKGPvb7KoXRyaR1U4ckNmy19VUs73UksSGSYTG/0/*)#pa8pyggg",
  "iswatchonly": false,
  "isscript": false,
  "iswitness": true,
  "witness_version": 0,
  "witness_program": "c435efcb40d70359f5b242316c0736579cb1d902",
  "pubkey": "0282b5b54f1c99e6d385f13517bb08a81cfc1ef1737596c503511fcb8cb2b499f0",
  "ischange": false,
  "timestamp": 1785578684,
  "hdkeypath": "m/84h/1h/0h/0/0",
  "hdseedid": "0000000000000000000000000000000000000000",
  "hdmasterfingerprint": "53a25ce0",
  "labels": [
    "classmate"
  ]
}
bitcoin@backend1:/$ 
```

## Evidence references

TODO: Link screenshots or describe the attached evidence.
The first screen shot shows me creating wallet and listing wallets created with the right RPC methods

The second screenshot generate newaddress for the wallet passing the -rpcwallet flag  and get the addressinfo of the address generated

The third screenshot shows the result of the test on lab02

## Explanation

TODO: Explain wallet context and the purpose of `-rpcwallet`.
