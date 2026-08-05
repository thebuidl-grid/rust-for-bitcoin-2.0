# Lab 09 — Multi-UTXO coin selection

## Commands used

cargo run --example lab09

Underlying bitcoin-cli RPCs invoked:
- createwallet "alice"
- getnewaddress "alice-funding"                          (wallet: alice)
- sendtoaddress <alice_address> 0.4  x3                  (wallet: miner)
- generatetoaddress 1 <miner_address>
- listunspent                                             (wallet: alice)
- getnewaddress "alice-payment"                           (wallet: receiver)
- sendtoaddress <receiver_address> 1                      (wallet: alice)
- generatetoaddress 1 <miner_address>
- getrawtransaction <spend_txid> 2

## Terminal output

alice address: bcrt1qfxg4cvndxmh2qywk5wa2r64a94xerx0uxe6a48
funding txids: ["416f88e403422b31005c91b2e4e558ce33509769dd2ca2516f0368543ac68ee4", "f87672117bbb97c02a1a03f32605c54ace2ed7952f62542ae54a6de22a472ade", "a038664b824d0e9f23c4d62277ebe19624691066af98d0a9a74d6cd22af70480"]
alice confirmed UTXO count: 3
  a038664b824d0e9f23c4d62277ebe19624691066af98d0a9a74d6cd22af70480:1 amount=0.4
  f87672117bbb97c02a1a03f32605c54ace2ed7952f62542ae54a6de22a472ade:0 amount=0.4
  416f88e403422b31005c91b2e4e558ce33509769dd2ca2516f0368543ac68ee4:1 amount=0.4
new receiver address: bcrt1qwwjjaspt9rpxzhe2td0l3gha3xax4fxq5u4wu6
spend txid: 7fdc2f5743f517dd5ccdeced40d91c3098c54098725e3674e7e55c2056de1673
spend input count: 3
payment output: DecodedOutput { vout: 1, value: 1.0, address: Some("bcrt1qwwjjaspt9rpxzhe2td0l3gha3xax4fxq5u4wu6"), script_pub_key_hex: "001473a52ec02b28c2615f2a5b5ff8a2fd89ba6aa4c0" }
change output:  Some(DecodedOutput { vout: 0, value: 0.1999448, address: Some("bcrt1qmq7x5fjfmv4lag99d3dz4sx3kl3ecse6qa2dlr"), script_pub_key_hex: "0014d83c6a2649db2bfea0a56c5a2ac0d1b7e39c433a" })
fee: 0.0000552
funding outpoints: ["a038664b824d0e9f23c4d62277ebe19624691066af98d0a9a74d6cd22af70480:1", "f87672117bbb97c02a1a03f32605c54ace2ed7952f62542ae54a6de22a472ade:0", "416f88e403422b31005c91b2e4e558ce33509769dd2ca2516f0368543ac68ee4:1"]

## Evidence references

alice confirmed UTXO count: 3
  a038664b824d0e9f23c4d62277ebe19624691066af98d0a9a74d6cd22af70480:1 amount=0.4
  f87672117bbb97c02a1a03f32605c54ace2ed7952f62542ae54a6de22a472ade:0 amount=0.4
  416f88e403422b31005c91b2e4e558ce33509769dd2ca2516f0368543ac68ee4:1 amount=0.4

spend input count: 3
## Explanation

Input combination happens when a wallet can't satisfy a payment amount with a single UTXO, so it selects and spends multiple UTXOs together as inputs in one transaction.
Change is what happens when the combined input value exceeds what you actually intend to send, the wallet creates a second output sending the leftover back to an address it controls so `input_total = payment_amount + change_amount + fee`.
Fee is just the residual left unassigned that the miner collects.
Privacy implication refers to the risk associated with combining inputs in one transaction because it can publicly link them to as associated with one entity