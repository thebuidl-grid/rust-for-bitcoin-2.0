# Demo transcript

Unedited output of `./scripts/demo.sh --reset` against a fresh regtest chain,
captured on Bitcoin Core v30.2.0.

Both seeds are generated at the start of the run, so a rerun produces the same
sequence of steps with different keys, addresses and transaction ids. The ids
quoted in `Readme.md` and used as fixtures in `src/raw.rs` come from this run.

```bash
./scripts/regtest-node.sh reset
./scripts/regtest-node.sh start
./scripts/demo.sh --reset
```

```text
removed existing wallet databases


=== Build ===
built ./target/release/rfbwallet


=== The node ===

$ ./target/release/rfbwallet node

node
  rpc url               http://127.0.0.1:18449
  authentication        user/password (user `rfb`)
  version               /Satoshi:30.2.0/
  chain                 regtest
  blocks                0
  headers               0
  best block            0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206
  in ibd                yes
  peers                 0
  fee estimate 6 blk    unavailable (normal on regtest)


=== Create the spending wallet (wpkh, BIP84) ===

$ ./target/release/rfbwallet init --descriptor wpkh

wallet created
  database              data/wallet.sqlite
  network               regtest
  descriptor            wpkh (BIP84)
  birthday              current node height (0)

descriptors (public, as persisted)
  external (receive)
    wpkh([43eb543c/84'/1'/0']tpubDCeP3HXrrm5q4WxnPfirieQqkpc8SuV6TSCv4gQUc2waeoAEqmg5dAc8SN1eaC214ugjbxD3NNJqEwoPXEU9YJxVAd5YQfdMVxsdws7Cha8/0/*)#3q86s9fl
  internal (change)
    wpkh([43eb543c/84'/1'/0']tpubDCeP3HXrrm5q4WxnPfirieQqkpc8SuV6TSCv4gQUc2waeoAEqmg5dAc8SN1eaC214ugjbxD3NNJqEwoPXEU9YJxVAd5YQfdMVxsdws7Cha8/1/*)#q5zmdse8

first receive address
  index                 0
  address               bcrt1q67m3kg4e5y9zd28m3fcnj2rdwav2s7su7a2gn9

The database holds public descriptors only. Keep RFB_MNEMONIC in your .env:
it is the only copy of the private keys, and every command that signs rebuilds
them from it at runtime.


=== Create the receiving wallet (tr, BIP86) from a second seed ===
generated a second seed into data/taproot.seed (gitignored)

$ RFB_WALLET_DB=data/taproot.sqlite rfbwallet init --descriptor tr

wallet created
  database              data/taproot.sqlite
  network               regtest
  descriptor            tr (BIP86)
  birthday              current node height (0)

descriptors (public, as persisted)
  external (receive)
    tr([abddb489/86'/1'/0']tpubDD1ZiTqaud8AWxPhGbSsuR4F4eWzrnCRR5U7uMYtQy72eTdHYiUBNHLUd2vCpfUUVpiWtYapVzqHTZpwX8ANFbDGoy2oY6ppuxCoAiCRiHo/0/*)#a2q5js5a
  internal (change)
    tr([abddb489/86'/1'/0']tpubDD1ZiTqaud8AWxPhGbSsuR4F4eWzrnCRR5U7uMYtQy72eTdHYiUBNHLUd2vCpfUUVpiWtYapVzqHTZpwX8ANFbDGoy2oY6ppuxCoAiCRiHo/1/*)#v79409y9

first receive address
  index                 0
  address               bcrt1p4k2dlxmknrv6f3vvwfq4xeee908hnjcm5v36877hvhyuy44vdauqxkgh7l

The database holds public descriptors only. Keep RFB_MNEMONIC in your .env:
it is the only copy of the private keys, and every command that signs rebuilds
them from it at runtime.


=== Fund the spending wallet ===

$ ./target/release/rfbwallet mine --blocks 101

mined
  blocks                101
  to address            bcrt1qxxyjkc20x3fj66qcrjfj5ejg6caauqnscveshl
  first block           026bdfc8a6ac787cec4c437c09c9003f4d4c30a7f392dbe16d86b4cafaf92212
  tip block             397f9dfa7d05ea427ce2792e501005c0235d08e66c70e02a51abc32e8177872a
  node height           101

Run `rfbwallet sync` to pull these blocks into the wallet.

$ ./target/release/rfbwallet sync

sync
  start height          0
  blocks applied        101
  tip height            101
  mempool txs seen      0
  mempool evictions     0

balance
  before                0 sat (0.00000000 BTC)
  after                 505000000000 sat (5050.00000000 BTC)
  spendable now         10000000000 sat (100.00000000 BTC)

$ ./target/release/rfbwallet balance

balance
  confirmed             10000000000 sat (100.00000000 BTC)
  trusted pending       0 sat (0.00000000 BTC)
  untrusted pending     0 sat (0.00000000 BTC)
  immature              495000000000 sat (4950.00000000 BTC)
  spendable now         10000000000 sat (100.00000000 BTC)
  total                 505000000000 sat (5050.00000000 BTC)

Immature funds are coinbase outputs. They need 100 confirmations before
they can be spent; on regtest, mine 100 more blocks.

$ ./target/release/rfbwallet utxos

unspent outputs
  outpoint                                                            sat         keychain            index  status
  9d2db688b726c5078dc31aef35d4a817136c93ae0456e3310d325d36df29be1b:0  5000000000  external (receive)  1      confirmed @ 1
  3c71eea4f99cf38d87c20bef74fa10618227e7e631ef0a2f6bd979ab1c4d5314:0  5000000000  external (receive)  1      confirmed @ 2
  e3767d661394d51e48f59dfee84d3845b31ff2a25d679b1663d0d10b262f6486:0  5000000000  external (receive)  1      immature @ 3 (99/100)
  b4cd90f295fb38cd22e0c2089051482161a1c8188de632879a35d0fcc1803ec8:0  5000000000  external (receive)  1      immature @ 4 (98/100)
  3e1973aaa4466b6cc298c07e9323d9ce09b156a884626df1b98573953d4e81d5:0  5000000000  external (receive)  1      immature @ 5 (97/100)
  f991de2f936c7ab83bbe21ccb0c877f96055e42b9ce282487857a82c553c55b0:0  5000000000  external (receive)  1      immature @ 6 (96/100)
  297ffd1db62f185ee89ab916ff5e4e3ef8dc65226a3a204cb9068310c032ba4b:0  5000000000  external (receive)  1      immature @ 7 (95/100)
  6b77287e0b31bb9918314372808f1aca51bc78e27c393faaa2e615c7f1a63e7f:0  5000000000  external (receive)  1      immature @ 8 (94/100)
  8f489afba134a985a18ac7fd50374a355b013c5c699739d98b2fe93b45ecfcd5:0  5000000000  external (receive)  1      immature @ 9 (93/100)
  0cb10097c35c9d6bc7f103c19cd552ebf4285687b6e1b79f46f3f72c459c9c3b:0  5000000000  external (receive)  1      immature @ 10 (92/100)
  09d95d854a4a47587bcb1e5eb5f8fee087098200194d4c3d97eff200df472b1b:0  5000000000  external (receive)  1      immature @ 11 (91/100)
  5654edd20f01c84b64799f2e752cac77cf6f212db254a96935112622df64884b:0  5000000000  external (receive)  1      immature @ 12 (90/100)
  80ee27e9ad090a0f7ae332a62d5bc7b7284aff909bcec88bad466acd074a84a5:0  5000000000  external (receive)  1      immature @ 13 (89/100)
  28ba465bc920746bbd8cd010fba6ae6804ce2aac1657e59f8d3f2f315b3bbba9:0  5000000000  external (receive)  1      immature @ 14 (88/100)
  1a19181c8214eb2ba0137cff0362d6b5b885e012eb55b230361016fcec8bff23:0  5000000000  external (receive)  1      immature @ 15 (87/100)

  showing               15 of 101 (pass --all for the rest)
  count                 101
  total                 505000000000 sat (5050.00000000 BTC)
  spendable             10000000000 sat (100.00000000 BTC)


=== Pay 1 BTC from the wpkh wallet to the tr wallet ===
recipient address: bcrt1p4k2dlxmknrv6f3vvwfq4xeee908hnjcm5v36877hvhyuy44vdauqxkgh7l

$ ./target/release/rfbwallet send --to bcrt1p4k2dlxmknrv6f3vvwfq4xeee908hnjcm5v36877hvhyuy44vdauqxkgh7l --amount 100000000 --fee-rate 2

transaction
  txid                  8bd94892811f83e975a59d35cf0c50e5a37804dc50e0105c38ef81b2d63220de
  recipient             bcrt1p4k2dlxmknrv6f3vvwfq4xeee908hnjcm5v36877hvhyuy44vdauqxkgh7l
  amount                100000000 sat (1.00000000 BTC)
  coin selection        branch-and-bound
  inputs                1
  outputs               2
  change                4899999695 sat (48.99999695 BTC)
  fee                   305 sat (0.00000305 BTC)
  effective fee rate    2.00 sat/vB (501 sat/kwu)
  weight                609 wu
  virtual size          153 vB

broadcast
  txid                  8bd94892811f83e975a59d35cf0c50e5a37804dc50e0105c38ef81b2d63220de
  relayed to            http://127.0.0.1:18449

Verify it independently with:
  rfbwallet verify 8bd94892811f83e975a59d35cf0c50e5a37804dc50e0105c38ef81b2d63220de


=== Confirm it and let both wallets discover it independently ===

$ ./target/release/rfbwallet mine --blocks 1

mined
  blocks                1
  to address            bcrt1qk3tvlav0eck8al5589tl7dw34cc450hfl07laz
  first block           5739a7f55841604b821fab4f2305494ab65e72f829f71b9b0e118b65e95a940f
  tip block             5739a7f55841604b821fab4f2305494ab65e72f829f71b9b0e118b65e95a940f
  node height           102

Run `rfbwallet sync` to pull these blocks into the wallet.

$ ./target/release/rfbwallet sync

sync
  start height          101
  blocks applied        1
  tip height            102
  mempool txs seen      0
  mempool evictions     0

balance
  before                504899999695 sat (5048.99999695 BTC)
  after                 509900000000 sat (5099.00000000 BTC)
  spendable now         14899999695 sat (148.99999695 BTC)

$ RFB_WALLET_DB=data/taproot.sqlite rfbwallet sync

sync
  start height          0
  blocks applied        102
  tip height            102
  mempool txs seen      0
  mempool evictions     0

balance
  before                0 sat (0.00000000 BTC)
  after                 100000000 sat (1.00000000 BTC)
  spendable now         100000000 sat (1.00000000 BTC)

$ RFB_WALLET_DB=data/taproot.sqlite rfbwallet utxos

unspent outputs
  outpoint                                                            sat        keychain            index  status
  8bd94892811f83e975a59d35cf0c50e5a37804dc50e0105c38ef81b2d63220de:0  100000000  external (receive)  0      confirmed @ 102

  count                 1
  total                 100000000 sat (1.00000000 BTC)
  spendable             100000000 sat (1.00000000 BTC)


=== Verify the payment from its raw bytes (rust-bitcoin, no BDK) ===

$ ./target/release/rfbwallet verify 8bd94892811f83e975a59d35cf0c50e5a37804dc50e0105c38ef81b2d63220de

decoded transaction
  source                bitcoind (getrawtransaction)
  raw size              234 B
  version               2
  lock time             101
  weight                609 wu
  virtual size          153 vB
  wtxid                 e104e206a3f2241998ab1e3d68a75edac4f776e3d05494ed8ee650aa297a7e22

txid check
  requested             8bd94892811f83e975a59d35cf0c50e5a37804dc50e0105c38ef81b2d63220de
  recomputed            8bd94892811f83e975a59d35cf0c50e5a37804dc50e0105c38ef81b2d63220de
  match                 yes

inputs
  #  spends                                                              sat         type                          signature
  0  9d2db688b726c5078dc31aef35d4a817136c93ae0456e3310d325d36df29be1b:0  5000000000  p2wpkh (segwit v0 key spend)  valid

outputs
  #  sat         type
  0  100000000   p2tr
  1  4899999695  p2wpkh

value
  inputs                5000000000 sat (50.00000000 BTC)
  outputs               4999999695 sat (49.99999695 BTC)
  fee                   305 sat (0.00000305 BTC)

Every signature was recomputed from the decoded bytes and verified against
the public key committed to by the previous output. No wallet state involved.


=== Send some back, spending a Taproot output ===
spender address: bcrt1q67m3kg4e5y9zd28m3fcnj2rdwav2s7su7a2gn9

$ RFB_WALLET_DB=data/taproot.sqlite rfbwallet send --to bcrt1q67m3kg4e5y9zd28m3fcnj2rdwav2s7su7a2gn9 --amount 40000000 --selection largest-first

transaction
  txid                  51b86dcf1fbe20424cb3aa2efb8339b59e854899ce91aa58ed9e0ef44cb733af
  recipient             bcrt1q67m3kg4e5y9zd28m3fcnj2rdwav2s7su7a2gn9
  amount                40000000 sat (0.40000000 BTC)
  coin selection        largest-first
  inputs                1
  outputs               2
  change                59999572 sat (0.59999572 BTC)
  fee                   428 sat (0.00000428 BTC)
  effective fee rate    3.02 sat/vB (754 sat/kwu)
  weight                568 wu
  virtual size          142 vB

broadcast
  txid                  51b86dcf1fbe20424cb3aa2efb8339b59e854899ce91aa58ed9e0ef44cb733af
  relayed to            http://127.0.0.1:18449

Verify it independently with:
  rfbwallet verify 51b86dcf1fbe20424cb3aa2efb8339b59e854899ce91aa58ed9e0ef44cb733af

$ ./target/release/rfbwallet mine --blocks 1

mined
  blocks                1
  to address            bcrt1qhdzh4uehn8w6wmyn54de9ly49836nyncdevm75
  first block           2966c0f396cce609782489cb87abbf56ebc5ad080efe8debb548678952c35d3e
  tip block             2966c0f396cce609782489cb87abbf56ebc5ad080efe8debb548678952c35d3e
  node height           103

Run `rfbwallet sync` to pull these blocks into the wallet.

$ ./target/release/rfbwallet sync

sync
  start height          102
  blocks applied        1
  tip height            103
  mempool txs seen      0
  mempool evictions     0

balance
  before                509900000000 sat (5099.00000000 BTC)
  after                 514940000428 sat (5149.40000428 BTC)
  spendable now         19939999695 sat (199.39999695 BTC)


=== Verify the Taproot key spend ===

$ ./target/release/rfbwallet verify 51b86dcf1fbe20424cb3aa2efb8339b59e854899ce91aa58ed9e0ef44cb733af

decoded transaction
  source                bitcoind (getrawtransaction)
  raw size              193 B
  version               2
  lock time             102
  weight                568 wu
  virtual size          142 vB
  wtxid                 1a8ee316ef68c243454ed390ee21638d576ade7e82772cdd36e8436b1bfa17b6

txid check
  requested             51b86dcf1fbe20424cb3aa2efb8339b59e854899ce91aa58ed9e0ef44cb733af
  recomputed            51b86dcf1fbe20424cb3aa2efb8339b59e854899ce91aa58ed9e0ef44cb733af
  match                 yes

inputs
  #  spends                                                              sat        type                      signature
  0  8bd94892811f83e975a59d35cf0c50e5a37804dc50e0105c38ef81b2d63220de:0  100000000  p2tr (taproot key spend)  valid

outputs
  #  sat       type
  0  59999572  p2tr
  1  40000000  p2wpkh

value
  inputs                100000000 sat (1.00000000 BTC)
  outputs               99999572 sat (0.99999572 BTC)
  fee                   428 sat (0.00000428 BTC)

Every signature was recomputed from the decoded bytes and verified against
the public key committed to by the previous output. No wallet state involved.


=== Persistence: a second sync has nothing left to do ===

$ ./target/release/rfbwallet sync

sync
  start height          103
  blocks applied        0
  tip height            103
  mempool txs seen      0
  mempool evictions     0

balance
  before                514940000428 sat (5149.40000428 BTC)
  after                 514940000428 sat (5149.40000428 BTC)
  spendable now         19939999695 sat (199.39999695 BTC)

$ ./target/release/rfbwallet info

wallet
  database              data/wallet.sqlite
  size on disk          152.0 KiB
  network               regtest
  descriptor            wpkh (BIP84)
  master fingerprint    43eb543c
  can sign              yes
  birthday height       0

keychains
  keychain            last revealed  next index  checksum
  external (receive)  3              4           3q86s9fl
  internal (change)   0              1           q5zmdse8

descriptors (public, as persisted)
  external (receive)
    wpkh([43eb543c/84'/1'/0']tpubDCeP3HXrrm5q4WxnPfirieQqkpc8SuV6TSCv4gQUc2waeoAEqmg5dAc8SN1eaC214ugjbxD3NNJqEwoPXEU9YJxVAd5YQfdMVxsdws7Cha8/0/*)#3q86s9fl
  internal (change)
    wpkh([43eb543c/84'/1'/0']tpubDCeP3HXrrm5q4WxnPfirieQqkpc8SuV6TSCv4gQUc2waeoAEqmg5dAc8SN1eaC214ugjbxD3NNJqEwoPXEU9YJxVAd5YQfdMVxsdws7Cha8/1/*)#q5zmdse8

chain state
  checkpoint height     103
  checkpoint hash       2966c0f396cce609782489cb87abbf56ebc5ad080efe8debb548678952c35d3e
  transactions          105
  unspent outputs       104


=== The same wallet without its seed ===

$ RFB_MNEMONIC= rfbwallet info

wallet
  database              data/wallet.sqlite
  size on disk          152.0 KiB
  network               regtest
  descriptor            wpkh (BIP84)
  master fingerprint    43eb543c
  can sign              no
  birthday height       0



=== Compare the two descriptor types ===

$ ./target/release/rfbwallet compare

wpkh vs tr, same seed

Both descriptors below come from the same mnemonic. They differ only in the
BIP43 purpose field and the script type, which is enough to make them
completely separate wallets with separate address sets.

wpkh (BIP84)
  account path          84'/1'/0'
  descriptor (public)
    wpkh([43eb543c/84'/1'/0']tpubDCeP3HXrrm5q4WxnPfirieQqkpc8SuV6TSCv4gQUc2waeoAEqmg5dAc8SN1eaC214ugjbxD3NNJqEwoPXEU9YJxVAd5YQfdMVxsdws7Cha8/0/*)#3q86s9fl
  address 0             bcrt1q67m3kg4e5y9zd28m3fcnj2rdwav2s7su7a2gn9
  script pubkey         0014d7b71b22b9a10a26a8fb8a7139286d7758a87a1c
  output size           31 B
  spend witness         107 wu (max)

tr (BIP86)
  account path          86'/1'/0'
  descriptor (public)
    tr([43eb543c/86'/1'/0']tpubDD87B5hsGFYGm9zkXptWC5sJXEg5QsmZyj195pnZQZytuz1SLhaMEAzb9GxcNJQmgbieP2Tt6YS4JXfD8pNQjGkn3aaynkH3AkVmWQtxMWP/0/*)#mfk9uj68
  address 0             bcrt1pdyh54zxp26krkgldhlhzlcstl9cj8un6qqppu0lp8yx55wxdph6qzw7n2l
  script pubkey         5120692f4a88c156ac3b23edbfee2fe20bf97123f27a00021e3fe1390d4a38cd0df4
  output size           43 B
  spend witness         66 wu (max)

summary
  descriptor  account path  output bytes  witness wu  address chars
  wpkh        84'/1'/0'     31            107         44
  tr          86'/1'/0'     43            66          64

Taproot pays 12 more bytes per output (a 32-byte x-only key against a 20-byte
hash) and gets them back on the spend: a key spend is one 64-byte Schnorr
signature, where wpkh needs a ~72-byte DER signature plus a 33-byte public key.
The witness column above is miniscript's own worst-case satisfaction weight.


=== Done ===
spend txid:  8bd94892811f83e975a59d35cf0c50e5a37804dc50e0105c38ef81b2d63220de
return txid: 51b86dcf1fbe20424cb3aa2efb8339b59e854899ce91aa58ed9e0ef44cb733af
```
