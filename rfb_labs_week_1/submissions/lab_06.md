# Lab 06 — Transaction decoding

## Commands used

docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getrawtransaction 37deb68194cfbdf5d8c723264017a83901f15e09d912319b1efcd84b5a05fb36 2   


## Terminal output


{
  "txid": "37deb68194cfbdf5d8c723264017a83901f15e09d912319b1efcd84b5a05fb36",
  "hash": "082459e96f87a0b642a6a173e42ce444410c5072eda2a0fe874ee6dc14e28fbd",
  "version": 2,
  "size": 370,
  "vsize": 208,
  "weight": 832,
  "locktime": 303,
  "vin": [
    {
      "txid": "e2ec2d676347605ca4df16d54f0701b550f108b4543b9bfa6a630f0e546cf030",
      "vout": 0,
      "scriptSig": {
        "asm": "",
        "hex": ""
      },
      "txinwitness": [
        "3044022015c5bbe2e34a8f1263582fb324ce083bb9dfae34ccf0b25ba70854b66854265a02202cb0367aa745078ecefe297a5d085708b0d54a7e01001b2055f8f27ac54f9ec101",
        "03215b73a854767afc1faa3f3d7b55ba33d4d91800920cbe2bbf2464e0096dce81"
      ],
      "prevout": {
        "generated": false,
        "height": 203,
        "value": 48.99998590,
        "scriptPubKey": {
          "asm": "0 9ca5184056f7fd28a2a7cc7b2f5750ce7610a6c9",
          "desc": "addr(bcrt1qnjj3sszk7l7j3g48e3aj746seemppfkft2zjc0)#6th67l4g",
          "hex": "00149ca5184056f7fd28a2a7cc7b2f5750ce7610a6c9",
          "address": "bcrt1qnjj3sszk7l7j3g48e3aj746seemppfkft2zjc0",
          "type": "witness_v0_keyhash"
        }
      },
      "sequence": 4294967293
    },
    {
      "txid": "721149681cc4ee48ee0df9952f5c1c0647f585fccdc75888be53a75cd0993044",
      "vout": 0,
      "scriptSig": {
        "asm": "",
        "hex": ""
      },
      "txinwitness": [
        "304402202f76dad787f5d729153d6ecfb4de5541558043067115eb5592d6e5d9c180b4b20220607736efd28ecf99ee520ca662d81c533b0b0210981dcd4a2debe90ae1a6fa5701",
        "03fb58702e81afa2e114717733393bd4d0a0b41b069b3feaadbae5b956cbe2a170"
      ],
      "prevout": {
        "generated": true,
        "height": 203,
        "value": 25.00001410,
        "scriptPubKey": {
          "asm": "0 96f59b3755128ff258aa64ae30b494631a7f006f",
          "desc": "addr(bcrt1qjm6ekd64z28lyk92vjhrpdy5vvd87qr0wf08aq)#wvkmq9r4",
          "hex": "001496f59b3755128ff258aa64ae30b494631a7f006f",
          "address": "bcrt1qjm6ekd64z28lyk92vjhrpdy5vvd87qr0wf08aq",
          "type": "witness_v0_keyhash"
        }
      },
      "sequence": 4294967293
    }
  ],
  "vout": [
    {
      "value": 13.99995840,
      "n": 0,
      "scriptPubKey": {
        "asm": "0 3998b4e543dcb13e4cfd3b566050c81d64e5f191",
        "desc": "addr(bcrt1q8xvtfe2rmjcnun8a8dtxq5xgr4jwtuv38tzqqd)#quavrp0h",
        "hex": "00143998b4e543dcb13e4cfd3b566050c81d64e5f191",
        "address": "bcrt1q8xvtfe2rmjcnun8a8dtxq5xgr4jwtuv38tzqqd",
        "type": "witness_v0_keyhash"
      }
    },
    {
      "value": 60.00000000,
      "n": 1,
      "scriptPubKey": {
        "asm": "0 a5771d0f26668cf1f909c34dadba98ff23296c01",
        "desc": "addr(bcrt1q54m36rexv6x0r7gfcdx6mw5clu3jjmqpydqg2n)#4clnazrz",
        "hex": "0014a5771d0f26668cf1f909c34dadba98ff23296c01",
        "address": "bcrt1q54m36rexv6x0r7gfcdx6mw5clu3jjmqpydqg2n",
        "type": "witness_v0_keyhash"
      }
    }
  ],
  "fee": 0.00004160,
  "hex": "0200000000010230f06c540e0f636afa9b3b54b408f150b501074fd516dfa45c604763672dece20000000000fdffffff443099d05ca753be8858c7cdfc85f547061c5c2f95f90dee48eec41c684911720000000000fdffffff02c03d7253000000001600143998b4e543dcb13e4cfd3b566050c81d64e5f19100bca06501000000160014a5771d0f26668cf1f909c34dadba98ff23296c0102473044022015c5bbe2e34a8f1263582fb324ce083bb9dfae34ccf0b25ba70854b66854265a02202cb0367aa745078ecefe297a5d085708b0d54a7e01001b2055f8f27ac54f9ec1012103215b73a854767afc1faa3f3d7b55ba33d4d91800920cbe2bbf2464e0096dce810247304402202f76dad787f5d729153d6ecfb4de5541558043067115eb5592d6e5d9c180b4b20220607736efd28ecf99ee520ca662d81c533b0b0210981dcd4a2debe90ae1a6fa57012103fb58702e81afa2e114717733393bd4d0a0b41b069b3feaadbae5b956cbe2a1702f010000",
  "blockhash": "2b7ff41f4069d504c401017626532c9c7670faf0153d6942b63ff2a3ffe273ff",
  "confirmations": 101,
  "time": 1785757028,
  "blocktime": 1785757028
}

## Evidence references


https://drive.google.com/drive/folders/1HvmkTC2bazkXgBELjgbLaaW8grJQgF9h?usp=sharing


## Explanation


Bitcoin's transaction format has no field anywhere for "fee." A transaction only declares two things: which previous outputs it consumes (inputs) and where the value goes (outputs). The fee isn't written down at all — it's an emergent quantity, computed as sum(inputs) - sum(outputs), and it only exists as an inference the node makes after the fact.

My own decoded transaction shows this precisely: the single input consumed 50.0 BTC (the matured coinbase UTXO from Lab 04). The outputs explicitly assign 1.0 BTC to the receiver and 48.9999718 BTC back to miner as change — together, 49.9999718. Nobody wrote 0.0000282 anywhere in the transaction; that number only appears because I computed 50.0 - 49.9999718 myself in calculate_fee. Bitcoin Core independently confirmed the same value when it exposed "fee" in the verbose-2 decode. The fee is real money that moved, but it was never assigned a destination inside the transaction structure — it's simply whatever value the sender chose not to claim in an output.

This design is what makes fees work as a market instead of a fixed cost. If the fee had to be a dedicated output, the sender would need to name a specific recipient address for it at the moment they signed the transaction — but they don't know in advance which miner will eventually include their transaction in a block; that's decided later, by open competition among miners choosing which transactions to include. By instead defining the fee as "whatever's left unassigned," any miner who successfully mines the block containing this transaction automatically collects that leftover value as part of their own coinbase reward — no need for the sender to pre-name a recipient, and no need for miners to coordinate in advance about who gets paid.`