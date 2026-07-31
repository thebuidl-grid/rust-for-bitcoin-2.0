# Lab 02 — Wallets and addresses

## Commands used

1. `createwallet` - Creates wallets named "miner" and "receiver"
2. `listwallets` - Lists all loaded wallets
3. `getnewaddress` - Generates labeled addresses (with `-rpcwallet` context)
4. `getaddressinfo` - Verifies address ownership (with `-rpcwallet` context)

### Connection Parameters
```bash
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass
```

### Wallet-Scoped Commands (Manual Examples)
```bash
# Create wallets (node-level, no -rpcwallet needed)
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass createwallet "miner"
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass createwallet "receiver"

# List wallets (node-level)
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass listwallets

# Generate address in miner wallet (wallet-scoped, needs -rpcwallet)
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=miner getnewaddress "mining"

# Generate address in receiver wallet (wallet-scoped, needs -rpcwallet)
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=receiver getnewaddress "classmate"

# Check address ownership (wallet-scoped, needs -rpcwallet)
bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=miner getaddressinfo "bcrt1q..."
```

## Terminal output

WALLET CREATION
{
  "name": "miner",
  "warning": ""
}

{
  "name": "receiver",
  "warning": ""
}


Step 2: Listing loaded wallets...

[
  "",
  "miner",
  "receiver"
]

  
ADDRESS GENERATION
   Miner address (label: mining): bcrt1qzm7xkwf5j47fdqysvflzu3h0lphs7ynky0tr2c
   Address uses bcrt1... regtest prefix
   Receiver address (label: classmate): bcrt1qslgwecw882ujdvg7vahk4z5y72fu6jaw58y7mm
   Address uses bcrt1... regtest prefix

VERIFYING OWNERSHIP
Miner address:-
  "address": "bcrt1qzm7xkwf5j47fdqysvflzu3h0lphs7ynky0tr2c",
  "scriptPubKey": "001416fc6b3934957c968090627e2e46eff86f0f1276",
  "ismine": true,
  "solvable": true,
  "desc": "wpkh([e311b891/84h/1h/0h/0/2]0244f3a6a0440f6cb96157c8d4f1d11ab6859b4c812c3beeb743cf08cb0bf39b18)#dg8sy3hc",
  "parent_desc": "wpkh([e311b891/84h/1h/0h]tpubDDqsDvXNe53A91T6Gz7wXxL9KwKZjQWmAFddHJnAYuuntKkRSpLwkACTw18FMjdws56dxo5ie4EcES1iNHrDDvomkV1ysBS4BYaKsuCLRs2/0/*)#n70thnsh",
  "iswatchonly": false,
  "isscript": false,
  "iswitness": true,
  "witness_version": 0,
  "witness_program": "16fc6b3934957c968090627e2e46eff86f0f1276",
  "pubkey": "0244f3a6a0440f6cb96157c8d4f1d11ab6859b4c812c3beeb743cf08cb0bf39b18",
  "ischange": false,
  "timestamp": 1785407058,
  "hdkeypath": "m/84h/1h/0h/0/2",
  "hdseedid": "0000000000000000000000000000000000000000",
  "hdmasterfingerprint": "e311b891",
  "labels": [
    "mining"
  ]

  Receiver Address:-
{
  "address": "bcrt1qslgwecw882ujdvg7vahk4z5y72fu6jaw58y7mm",
  "scriptPubKey": "001487d0ece1c73ab926b11e676f6a8a84f293cd4bae",
  "ismine": false,
  "solvable": false,
  "iswatchonly": false,
  "isscript": false,
  "iswitness": true,
  "witness_version": 0,
  "witness_program": "87d0ece1c73ab926b11e676f6a8a84f293cd4bae",
  "ischange": false,
  "labels": [
  ]
}

## Evidence references

### Screenshots Location
All screenshots are stored in `submissions/screenshots/`

1. **Wallet Creation**: `screenshots/lab02_wallets_created.png`
   - Shows both miner and receiver wallets loaded
   - Output from listwallets command

2. **Address Generation**: `screenshots/lab02_addresses.png`
   - Miner address with "mining" label
   - Receiver address with "classmate" label
   - Both showing bcrt1... prefix

3. **Rust Program Output**: `screenshots/lab02_rust_output.png`
   - Terminal showing `cargo run --example lab02_demo` execution
   - All verification steps passing

## Explanation

### Why wallet-scoped calls need `-rpcwallet`

Bitcoin Core can manage multiple wallets simultaneously. When an RPC method operates on wallet-specific data (like generating addresses, checking balances, or creating transactions), Bitcoin Core needs to know which wallet to use.


The `-rpcwallet` parameter tells Bitcoin Core to execute the RPC in the context of the specified wallet, ensuring operations like address generation, balance queries, and transaction creation use the correct wallet's keys and UTXO set.



