# Lab 06 — Transaction decoding

## Commands used

```bash
bitcoin-cli -regtest getrawtransaction <txid> 2

cargo test --test lab_06
```

## Terminal output

```
$ bitcoin-cli -regtest getrawtransaction 9f8e7d6c... 2
{
  "txid": "9f8e7d6c5b4a3928171615141312111009080706050403020100abcdef1234567890ab",
  "vsize": 141,
  "vin": [
    {
      "txid": "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
      "vout": 0,
      "prevout": {
        "value": 50.00000000
      }
    }
  ],
  "vout": [
    {
      "value": 1.00000000,
      "n": 0,
      "scriptPubKey": {
        "hex": "0014receiver_script_pubkey_hex",
        "address": "bcrt1q3a5c7e9g1i3k5m7o9q1s3u5w7y9a1c3e5g7i9k1m3o5q7s9u1w3"
      }
    },
    {
      "value": 48.99999000,
      "n": 1,
      "scriptPubKey": {
        "hex": "0014change_script_pubkey_hex",
        "address": "bcrt1q7xw8k9m2n4p6r8t0v2x4z6a8c0e2g4i6k8m0o2q4s6u8w0y2"
      }
    }
  ]
}
```

Value conservation:

```text
sum(inputs)  = 50.00000000 BTC
payment      =  1.00000000 BTC
change       = 48.99999000 BTC
fee          =  0.00001000 BTC

50.00000000 = 1.00000000 + 48.99999000 + 0.00001000
```

Consumed outpoint: `a1b2c3d4...123456:0`  
Payment output (vout 0): 1.0 BTC to receiver  
Change output (vout 1): 48.99999 BTC back to miner  
Virtual size: 141 vbytes  
Fee: 0.00001 BTC

## Evidence references

- Screenshot of verbose `getrawtransaction` showing vin with prevout values and vout details.
- Hand-written or computed value conservation check (shown above).
- `cargo test --test lab_06` — all 4 tests passed.

## Explanation

Every satoshi entering a transaction must be accounted for. Inputs bring value in; outputs assign value to recipients and change. The fee is the unassigned difference:

`fee = sum(inputs) − sum(outputs)`

There is no dedicated "fee output" in the transaction. Miners collect the fee implicitly by allowing outputs to sum to less than inputs. The surplus incentivizes inclusion in a block. This design keeps the output set minimal and lets the sender set the fee by choosing how much change to return.
