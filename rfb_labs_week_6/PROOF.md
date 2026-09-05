# Proof of a working transaction (regtest)

A real, end-to-end run against a live `bitcoind` (Docker image
`ruimarinho/bitcoin-core:24`, regtest, RPC on `127.0.0.1:18443`). All addresses, txids,
and balances below are real outputs from this run, not illustrative.

## 1. Init the wallet (BIP84 / `wpkh`)

```
$ cargo run -- init
Generated a new mnemonic and saved it to .env.
Keep that file out of version control — it is the wallet's private key material.
Network:            regtest
Descriptor type:    Wpkh
Account:            0
Wallet database:    wallet.sqlite
First receive address: bcrt1q0p7t9nc69reheg8agzpumlvgqgxngk62xshmfy
```

## 2. Fund it from a node-side wallet, and mine a confirmation

```
$ bitcoin-cli -regtest -rpcwallet=miner sendtoaddress bcrt1q0p7t9nc69reheg8agzpumlvgqgxngk62xshmfy 1.5
f7a18a537195971a51ca4068aa009a4f0948e670f91726a0269e3d06096d9cd7

$ bitcoin-cli -regtest -rpcwallet=miner generatetoaddress 1 <miner-address>
["62202920d53e03da6d8c764e147b4dc9a5f960951bcb07dad18cc75e0c542a04"]
```

## 3. Sync and check balance/UTXOs

```
$ cargo run -- sync
Applied 102 new block(s); wallet tip is now at height 102.
Balance after sync: 1.50000000 BTC

$ cargo run -- balance
Confirmed:          1.50000000 BTC
Trusted pending:    0 BTC
Untrusted pending:  0 BTC
Immature (coinbase):0 BTC
Total:              1.50000000 BTC

$ cargo run -- utxos
f7a18a537195971a51ca4068aa009a4f0948e670f91726a0269e3d06096d9cd7:0  1.50000000 BTC  External #0  Confirmed { anchor: ConfirmationBlockTime { block_id: BlockId { height: 102, hash: 62202920d53e03da6d8c764e147b4dc9a5f960951bcb07dad18cc75e0c542a04 }, ... } }
1 UTXO(s) total.
```

## 4. Construct, sign, and broadcast a transaction

```
$ cargo run -- send bcrt1qklluus3j8j9qrupfckdqf944vag6xn43wk55dt 10000
Broadcast transaction a372038c4dfd0b4524c0fe4f497216a4c68c0d232f1bc3dca886180664c6b559
Paid 10000 sats to bcrt1qklluus3j8j9qrupfckdqf944vag6xn43wk55dt
```

Mined a block, then verified directly against the node — **independent of our own
wallet's view**:

```
$ bitcoin-cli -regtest getrawtransaction a372038c4dfd0b4524c0fe4f497216a4c68c0d232f1bc3dca886180664c6b559 true
{
  "txid": "a372038c4dfd0b4524c0fe4f497216a4c68c0d232f1bc3dca886180664c6b559",
  "vin": [{ "txid": "f7a18a53...96d9cd7", "vout": 0, "txinwitness": [<sig>, <pubkey>] }],
  "vout": [
    { "value": 0.00010000, "address": "bcrt1qklluus3j8j9qrupfckdqf944vag6xn43wk55dt" },
    { "value": 1.49989859, "address": "bcrt1q3n9k6jsh0d56g42pl7sxakry4t2824e0pjg9jw" }
  ],
  "blockhash": "5397d092cc3618dbce652289e2960a45c7f8f945481cd11b465e183139b7b76a",
  "confirmations": 1
}
```

The transaction spends our own funded UTXO, pays the requested 10,000 sats to the
destination, sends the ~1.49989859 BTC change to a fresh address on our own *internal*
keychain, and is mined into block height 103.

## 5. Persistence survives a restart

Each `cargo run` above is a **separate process** re-opening `wallet.sqlite` from
scratch. Re-running after the send, with no state carried over in memory:

```
$ cargo run -- balance
Confirmed:          1.49989859 BTC
Total:              1.49989859 BTC

$ cargo run -- utxos
a372038c...b559:1  1.49989859 BTC  Internal #0  Confirmed { ... height: 103 ... }
1 UTXO(s) total.
```

The wallet correctly reconstructed its full synced state — including recognizing its
own change output — from disk alone.

## 6. Stretch goal: the same flow on a BIP86 Taproot (`tr`) wallet

```
$ cargo run -- --descriptor-kind tr --db-path wallet-tr.sqlite init
First receive address: bcrt1pwfe9epx9tznyvjl5pf02nzldhht9a8cq3pkuhu08mwdkq5mglmsqp2e7f3

$ bitcoin-cli -regtest -rpcwallet=miner sendtoaddress bcrt1pwfe9...mglmsqp2e7f3 0.2
265e6a1d9d6df7d11973946166dcd05a61209426c1301e919cb36e42b5f413a9
$ bitcoin-cli -regtest -rpcwallet=miner generatetoaddress 1 <miner-address>

$ cargo run -- --descriptor-kind tr --db-path wallet-tr.sqlite balance
Confirmed: 0.20000000 BTC

$ cargo run -- --descriptor-kind tr --db-path wallet-tr.sqlite send <addr> 5000
Broadcast transaction c169288b0fcd921a8bae0b4a718337727667ae3f60b6832851b7bc5ba90436bf

$ bitcoin-cli -regtest getrawtransaction c169288b...36bf true | grep -E "type|confirmations"
"type": "witness_v1_taproot"
"confirmations": 1
```

Same code path, same commands, one flag changed — a Taproot key-path spend, signed and
confirmed on chain.
