# Lab 08 — Block security

## Commands used
- `bitcoin-cli -regtest getblockheader <blockhash>` backs `get_block_header`
- `bitcoin-cli -regtest -rpcwallet=receiver gettransaction <txid>` backs `get_confirmations` (before)
- `bitcoin-cli -regtest -rpcwallet=miner getnewaddress "miner-address-3"` new mining address
- `bitcoin-cli -regtest generatetoaddress 5 <address>` — backs `mine_additional_blocks`
- `bitcoin-cli -regtest -rpcwallet=receiver gettransaction <txid>` backs `get_confirmations` (after)

These compose into `build_security_report`, which reads the block header and
initial confirmation count, mines 5 additional blocks, then re-reads the
confirmation count to prove it advances by exactly the number of blocks mined.

## Terminal output

$ bitcoin-cli -regtest getblockheader 66c50d2f8574457dcef9ce7effc055f7ba5f2aab9fa4395ae4ae9ba9e602df65
{
"hash": "66c50d2f8574457dcef9ce7effc055f7ba5f2aab9fa4395ae4ae9ba9e602df65",
"confirmations": 1,
"height": 689,
"version": 536870912,
"versionHex": "20000000",
"merkleroot": "d2d625deac53d7237dfb71f6a6472b7db1d41f37f9946092f269d69955aa746b",
"time": 1785408427,
"mediantime": 1785311749,
"nonce": 0,
"bits": "207fffff",
"difficulty": 4.656542373906925e-10,
"chainwork": "0000000000000000000000000000000000000000000000000000000000000564",
"nTx": 2,
"previousblockhash": "58e0954b1412ff585b4dc12e99577729cc552498032efb4afccc1a916556faf6"
}$ bitcoin-cli -regtest -rpcwallet=receiver gettransaction c5c0505f1800e8107d058e5823fb414fdd9c73560cb483f7ef984a31aa82ea54
{
"confirmations": 1,
"blockhash": "66c50d2f8574457dcef9ce7effc055f7ba5f2aab9fa4395ae4ae9ba9e602df65",
...
}$ bitcoin-cli -regtest -rpcwallet=miner getnewaddress "miner-address-3"
bcrt1qr4qxwpd9esqfa54flt2nrgurtgygjctutu2kvu

$ bitcoin-cli -regtest generatetoaddress 5 bcrt1qr4qxwpd9esqfa54flt2nrgurtgygjctutu2kvu
[
"40d80d8f1266fce8cf0235a43dd464e1786344027daa136167ad2e5a47ced5b3",
"0e9f280efc26b238fdbb4a79ce4464d81e7e6e11bc4593bfdcffc61b4befe814",
"61845ad120ad37fc4109b10377f17e3ddb1bbe0255209f9f35e0024c9804d865",
"470f517436340a048697f5b0456311fcf6d1c5ea60b734e4e950fdd409f1a015",
"1bb20e1e65a194dd46fa50f1f64752a43c6516a357f2f6daa920544bd2302820"
]

$ bitcoin-cli -regtest -rpcwallet=receiver gettransaction c5c0505f1800e8107d058e5823fb414fdd9c73560cb483f7ef984a31aa82ea54
{
"confirmations": 6,
"blockhash": "66c50d2f8574457dcef9ce7effc055f7ba5f2aab9fa4395ae4ae9ba9e602df65",
"blockheight": 689,
...
}


## Evidence references
Captured directly from the local regtest node, continuing from the same
transaction used in Labs 05–07 (confirmed in block 689). Mining exactly 5
additional blocks on top of that block raised the transaction's confirmation
count from 1 to 6, one increment per block mined — proving confirmation depth
tracks the number of blocks stacked on top of the confirming block.

## Explanation (co-authored by Claude)

Every block header contains a previousblockhash field, which is literally the hash of the prior block this is what "chains" blocks together into a single linear history. Because each block's hash depends on its own contents and the previous block's hash, you can't alter an old block without changing its hash, which would break the link to every block mined after it. This is the structural basis of blockchain immutability: rewriting history requires re-mining every subsequent block too.

The merkleroot field is a single hash that summarizes every transaction in the block. It's built by repeatedly hashing pairs of transaction hashes together until only one remains. This lets anyone verify that a specific transaction is included in a block's set without needing the full list of every transaction and any change to any transaction in the block would produce a completely different merkle root, so it also protects transaction data from tampering.

bits encodes the difficulty target the block's hash had to satisfy, and nonce is the value miners repeatedly change while searching for a hash that meets that target this trial-and-error process is proof of work. On this regtest node, the difficulty is set essentially to zero (bits: "207fffff", the easiest possible target), which is why blocks here can be mined instantly rather than requiring real computational effort, unlike mainnet.

Confirmation depth is simply how many blocks have been mined on top of the block containing a transaction. Each additional block adds more accumulated proof of work sitting on top of that transaction, making it progressively more expensive for anyone to rewrite the chain far enough back to undo it since doing so would mean re-doing the proof of work for every block from that point forward, faster than the rest of the network is adding new blocks. This is exactly what the evidence shows: mining 5 more blocks moved the transaction's confirmation count from 1 to 6, meaning it's now buried under five more blocks' worth of security than it was a moment before, which is why more confirmations are generally treated as more final/irreversible.
