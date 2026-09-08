# Lab 06 — Transaction decoding

## Commands used

The command was used to get raw transaction id 
`btc getrawtransaction 77fc470ebfc55a6ed118923afe58621b246c1748b641c7e515aa7e44a48f8ebc 2` # Used the transaction id from lab 6

## Terminal output

The output was as:
```
└─$ btc getrawtransaction 77fc470ebfc55a6ed118923afe58621b246c1748b641c7e515aa7e44a48f8ebc 2
{
  "txid": "77fc470ebfc55a6ed118923afe58621b246c1748b641c7e515aa7e44a48f8ebc",
  "hash": "93e8576e91766dfd97433c1169e1bc1dc5348eb2905734749f18feda39be9b33",
  "version": 2,
  "size": 222,
  "vsize": 141,
  "weight": 561,
  "locktime": 439,
  "vin": [
    {
      "txid": "39778cb2dd37cc26dcd7c579aac57c9b1a9cd23e68f693564f8a7e46a1afec7b",
      "vout": 0,
      "scriptSig": {
        "asm": "",
        "hex": ""
      },
      "txinwitness": [
        "304402207257bf173798a1083affa31583f1d8ddb362fa6ea2778676695e2a6f6daed5f102203f3ae5772cc3f193f9db4360e9f1e513b8ba45a908fa5c7835a1744c8ac483a501",
        "027364fc83502ceb92c9201132a7aff61f512941f28ecf2d9497a578ade91e69d4"
      ],
      "sequence": 4294967293
    }
  ],
  "vout": [
    {
      "value": 11.49997180,
      "n": 0,
      "scriptPubKey": {
        "asm": "0 9f23f1230f875720204208618156ccf28d482033",
        "desc": "addr(bcrt1qnu3lzgc0satjqgzzppscz4kv72x5sgpnpcfa0p)#6a48skm2",
        "hex": "00149f23f1230f875720204208618156ccf28d482033",
        "address": "bcrt1qnu3lzgc0satjqgzzppscz4kv72x5sgpnpcfa0p",
        "type": "witness_v0_keyhash"
      }
    },
    {
      "value": 1.00000000,
      "n": 1,
      "scriptPubKey": {
        "asm": "0 a39dc67794b7b4ea2f6ba08fb805fa31fcda2c94",
        "desc": "addr(bcrt1q5wwuvau5k76w5tmt5z8msp06x87d5ty53lzafh)#qa9d0xwf",
        "hex": "0014a39dc67794b7b4ea2f6ba08fb805fa31fcda2c94",
        "address": "bcrt1q5wwuvau5k76w5tmt5z8msp06x87d5ty53lzafh",
        "type": "witness_v0_keyhash"
      }
    }
  ],
  "hex": "020000000001017becafa1467e8a4f5693f6683ed29c1a9b7cc5aa79c5d7dc26cc37ddb28c77390000000000fdffffff027c908b44000000001600149f23f1230f875720204208618156ccf28d48203300e1f50500000000160014a39dc67794b7b4ea2f6ba08fb805fa31fcda2c940247304402207257bf173798a1083affa31583f1d8ddb362fa6ea2778676695e2a6f6daed5f102203f3ae5772cc3f193f9db4360e9f1e513b8ba45a908fa5c7835a1744c8ac483a50121027364fc83502ceb92c9201132a7aff61f512941f28ecf2d9497a578ade91e69d4b7010000"
}
```

## Evidence references

Attached the evidence as follows
![ProjectScreenshot](evidence/Lab%206.png)

## Explanation
Value conservation: summing the outputs from this transaction —
11.49997180 + 1.00000000 = 12.49997180 BTC — compared against the single
input's value (12.5 BTC, the matured coinbase UTXO from Lab 04) gives a
difference of 12.5 - 12.49997180 = 0.0000282 BTC. This matches exactly
the fee reported by `gettransaction` (`-0.00002820`), confirming
`inputs = outputs + fee`.

The fee has no dedicated output because it isn't sent anywhere by the
transaction itself — it's simply whatever value is left unclaimed once
all outputs are accounted for. Miners are entitled to collect this
leftover amount by including it in the coinbase reward of whichever
block confirms the transaction, so the fee is implicit rather than an
explicit line item: `fee = sum(inputs) - sum(outputs)`, calculated by
subtraction, not paid to a named address.
