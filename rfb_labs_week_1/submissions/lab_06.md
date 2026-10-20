# Lab 06 — Transaction decoding

## Commands used

bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass generatetoaddress 1 bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn
cargo run --example lab06

Underlying bitcoin-cli RPC invoked by the program:
- getrawtransaction a9e5849b95b19d9c08218953eeb0475c75b8b856f5838615bd37f37f6056647b 2

## Terminal output

txid:  a9e5849b95b19d9c08218953eeb0475c75b8b856f5838615bd37f37f6056647b
vsize: 141
inputs:
  8efa1967e9a59084cfbba48f83d8d503c8c09c9797ef0c2d88d926f00c961f39:0 value=50
outputs:
  vout=0 value=48.9999718 address=Some("bcrt1qeurr8vl42zksm65ysmqfa8u8c6ecdlc34syrm9") script_pub_key_hex=0014cf0633b3f550ad0dea8486c09e9f87c6b386ff11
  vout=1 value=1 address=Some("bcrt1q6nlqtswveesh573tml9mrwlczn66vfu0sjnaqn") script_pub_key_hex=0014d4fe05c1ccce617a7a2bdfcbb1bbf814f5a6278f
consumed outpoints: ["8efa1967e9a59084cfbba48f83d8d503c8c09c9797ef0c2d88d926f00c961f39:0"]
payment output: DecodedOutput { vout: 1, value: 1.0, address: Some("bcrt1q6nlqtswveesh573tml9mrwlczn66vfu0sjnaqn"), script_pub_key_hex: "0014d4fe05c1ccce617a7a2bdfcbb1bbf814f5a6278f" }
change output:  Some(DecodedOutput { vout: 0, value: 48.9999718, address: Some("bcrt1qeurr8vl42zksm65ysmqfa8u8c6ecdlc34syrm9"), script_pub_key_hex: "0014cf0633b3f550ad0dea8486c09e9f87c6b386ff11" })
fee: 0.0000282
sum(inputs) = 50  ==  payment(1) + change(48.9999718) + fee(0.0000282) = 50

## Evidence references

bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass getrawtransaction a9e5849b95b19d9c08218953eeb0475c75b8b856f5838615bd37f37f6056647b 2

{
  "txid": "a9e5849b95b19d9c08218953eeb0475c75b8b856f5838615bd37f37f6056647b",
  "hash": "455221de0e7da1fe3c5c21338b847063e0a55eb0f5c25e92a136554fc5a98aa5",
  "version": 2,
  "size": 222,
  "vsize": 141,
  "weight": 561,
  "locktime": 102,
  "vin": [
    {
      "txid": "8efa1967e9a59084cfbba48f83d8d503c8c09c9797ef0c2d88d926f00c961f39",
      "vout": 0,
      "scriptSig": {
        "asm": "",
        "hex": ""
      },
      "txinwitness": [
        "3044022075421bac4382b4f9b4a41fef21d0cd5b767700217b9fec96f86605faf3e77da102202e788eb7842d74bf2e7d245e5161b6ab2dac4ec95c02d4b9e3b4cae74806639c01",
        "039283c2d8171581dd2631c7d592313de8d722e9a954926180cf6fe02716d5048b"
      ],
      "sequence": 4294967293
    }
  ],
  "vout": [
    {
      "value": 48.99997180,
      "n": 0,
      "scriptPubKey": {
        "asm": "0 cf0633b3f550ad0dea8486c09e9f87c6b386ff11",
        "desc": "addr(bcrt1qeurr8vl42zksm65ysmqfa8u8c6ecdlc34syrm9)#0k0hwcrs",
        "hex": "0014cf0633b3f550ad0dea8486c09e9f87c6b386ff11",
        "address": "bcrt1qeurr8vl42zksm65ysmqfa8u8c6ecdlc34syrm9",
        "type": "witness_v0_keyhash"
      }
    },
    {
      "value": 1.00000000,
      "n": 1,
      "scriptPubKey": {
        "asm": "0 d4fe05c1ccce617a7a2bdfcbb1bbf814f5a6278f",
        "desc": "addr(bcrt1q6nlqtswveesh573tml9mrwlczn66vfu0sjnaqn)#hwajkfk8",
        "hex": "0014d4fe05c1ccce617a7a2bdfcbb1bbf814f5a6278f",
        "address": "bcrt1q6nlqtswveesh573tml9mrwlczn66vfu0sjnaqn",
        "type": "witness_v0_keyhash"
      }
    }
  ],
  "hex": "02000000000101391f960cf026d9882d0cef97979cc0c803d5d8838fa4bbcf8490a5e96719fa8e0000000000fdffffff02fc05102401000000160014cf0633b3f550ad0dea8486c09e9f87c6b386ff1100e1f50500000000160014d4fe05c1ccce617a7a2bdfcbb1bbf814f5a6278f02473044022075421bac4382b4f9b4a41fef21d0cd5b767700217b9fec96f86605faf3e77da102202e788eb7842d74bf2e7d245e5161b6ab2dac4ec95c02d4b9e3b4cae74806639c0121039283c2d8171581dd2631c7d592313de8d722e9a954926180cf6fe02716d5048b66000000"
}

## Explanation

Value conservation means the sum of a tx's input amounts must be greater or equal to the sum of its output amounts, with its proof being, for a tx with inputs summing to `total_in` and outputs summing to `total_out`, the rule `total_in >= totalout` must hold, or every full node rejects the tx as invalid. and `fee = total_in - total_out`.the fee has no dedicated output because whichever miner includes the tx in a block is entitled to collect that leftover difference as part of their coinbase reward.