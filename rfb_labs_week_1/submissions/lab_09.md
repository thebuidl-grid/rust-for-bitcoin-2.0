# Lab 09 — Multi-UTXO coin selection

## Commands used

TODO: Record funding, confirmation, spending, and decoding commands.

# ==========================================
# 1. FUNDING: Mine coins to the wallet
# ==========================================
echo "=== 1. FUNDING ==="
FUND_ADDR=$(docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 getnewaddress)
# Mine 101 blocks to mature the coinbase reward
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass generatetoaddress 101 "$FUND_ADDR"
echo "Funded wallet with 101 blocks."

# ==========================================
# 2. CONFIRMATION: Verify balance & UTXOs
# ==========================================
echo -e "\n=== 2. CONFIRMATION STATUS ==="
# Check confirmed balance
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 getbalances
# List specific UTXOs created by mining
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 listunspent | jq '.[0:2]' # Shows first 2 UTXOs

# ==========================================
# 3. SPENDING: Send a transaction
# ==========================================
echo -e "\n=== 3. SPENDING ==="
SPEND_ADDR=$(docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 getnewaddress)
TXID=$(docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 -named sendtoaddress address="$SPEND_ADDR" amount=10)
echo "Spent 10 BTC. TXID: $TXID"

# Check mempool before confirmation
echo "Mempool contains TX:"
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getrawmempool | grep "$TXID" || echo "Not found in mempool (maybe already mined?)"

# ==========================================
# 4. MINING FOR CONFIRMATION (Optional but recommended)
# ==========================================
echo -e "\n=== Confirming Transaction ==="
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass generatetoaddress 1 "$FUND_ADDR"

# ==========================================
# 5. DECODING: Inspect the spent transaction
# ==========================================
echo -e "\n=== 4. DECODING TRANSACTION ==="
# Verbose decode (Level 2) shows inputs, outputs, and block confirmation
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getrawtransaction "$TXID" 2

# ==========================================
# 6. FINAL STATE
# ==========================================
echo -e "\n=== FINAL BALANCE ==="
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 getbalances   

## Terminal output

TODO: Show Alice's three UTXOs and the combined transaction inputs and outputs.

=== 1. FUNDING ===
[
  "57a1d82efb2c5eb0dc0c34d42b56b00123b5d4f3748a10c6637b70bb42fdd150",
  "131cc8feb57dfb32967952685b47339c2bcbc617d7f53be355d6eb756df36f4c",
  "339bc4f6fdfce4bb947df7826fce248b308bf67419ea430de75546c3b4b7de92",
  "2637def2795c405efe6e0e6afddfd9cfe5be7196e92c1511fba4d68cb7e5f90e",
  "3b234024b99dce30811fead57b2ec66b17e8b5188a48ee146ae0591b387bb7f1",
  "59dbd886c1a6c47a40fc98452056f1560405b55cdda23b4c3d350cd2ef62b7bb",
  "393c9febbef168c883e7ec258884d520632ae2698276e0001b281a33ad647db6",
  "594d6c3f1c6089c0a84cef367a30a74eb84ff123ae2ad0c3982d2c6c7a2ac6f9",
  "175a98e9e828cef99554d0f98a53b01d1d7ec74d0bada20e2fafdfabb2312bc2",
  "6068200645eab227b86873453a6df8a9c82d3c6fc00ddda64cee8cb32ea039c4",
  "5460eb74478d62192f1d3e1a43159dfbeb846f906c5240b37538faf710563fa2",
  "230a37585ddfec018e564c10ebdf68bc6b537fb43b192eb11baf6f1bc41b1e9f",
  "73848b6f0ac0b8ad88c2b7a28235154875ff74e9df7485c49b9af6b92345100c",
  "1fe65844bb895a5f5d0a583cae56d8915c828933c9afb911d4f41c890d5a7225",
  "3535e739048636a0283c75f4b1d1c717158cd311a1a6d5ca7b51cbd707a38bea",
  "4015f686ca22b545ef53f591e0c087a1803035836b82c68bedc20a7fc7c2e288",
  "3b851dbe5676e6c89b6e4087e1d4b9818adb13fa6aee4b0d024fcb493d6b1f39",
  "5aef201e5d377034ee722e0b72ea9fe1b0c4d4aa9ef98d5389ebb5109da8a9ea",
  "2b6f9bf3ef390196b881b024481d3a21f5a1ffcf87854ea3d22749b14b44bfd6",
  "71f36cf8eb641339b97db14f0da364ba93d9795918061ac7f309370332f4f177",
  "06b092cf655f8eddaddf7316e45e58874ac0ee14994136ef793e3a461e8179b4",
  "37ef8632a30891cbf8678cb03c5cd8bda1b29d207dc7b2dcd16c642bd7a67a18",
  "25e8b82a4045bda656ae1ce059432d9ffa0122175fa4fd79b6d587381269c6cb",
  "4506fd23b9bcb1f55482305e812c90492fb4395a56d692687b5d63e55ea3de62",
  "3f11a0f2da8f9d503d696917296ccc29b35c7954a12a6f6778e73f5ebb1caff9",
  "7ff9fca6c7f40fa79542219359caa3a5d7585be0a9c41ff9fe11a449fe2c319f",
  "01dd9bdd7f893b5879915c9cc88c51a5e075d366855035c00cda2ec6ad55ed9d",
  "5c188180d2bc37f2588820c3043987ed516c1897494d5d8cc6a23883d366acc5",
  "664a21d1d368b5e3086f882f9e916800a50bfabd1d7300495797ac068af11649",
  "007e9b54a17ab0134ace7a8166f5cb0df98b08187843c2776c1402a28e276679",
  "71d1802afed3100eb7f0dcc89ffb08d9dc2ca1c5322a8f0ee3376a1ec69ffd6e",
  "768cd8bcb03e2c9c49201358248b5d6102e3070080a51875527a933eedc153a6",
  "0e114fdad0611afcd4195a32ae1e96707c5bd6e424712dc28b985a2f02cd168b",
  "290f026e0da14d500e7eda3c53a4e653f530477ba24edbfc75273b6188004157",
  "58cd5a5410e852c3136c1b45dba0445af773ee506f7385ee7859272832b60797",
  "43544732ff8f8e71e3127aca02ff21c5b043d4edf0c0adaa83a9e826fa287b6a",
  "6968d2202c55fa8f34db8f6337f3dd8a379cdca2aa7c0bd6fa7cd34f30c66ec0",
  "4ab12057101bf291b73649b87aaa9d5f6b6c9329ac696bdca00d995cd28d6ac5",
  "5bd641639cb0cb0f6a8d9339cff4ee627e1746f9b610446f75140c42316f272a",
  "5fae49325ad59c7ab1969034aed075629c3e26e3c0aac46a7503be81b20d1f9d",
  "2ad0f3e84552db5eadf3a0872dc5bb136c7a8c3ffd59b8f5de534b6ac68135d1",
  "59a2d9d0a8cc1360d88f2faf8d89ec937de833730aa1f6abdf5b4f44a796f8a2",
  "26bb7a7512df5ba9beb3bb5a6b98689f921e897d190ddd9dbec2205a996e053b",
  "68ce93999d332d62aeea2b9d5de3573081612793c2b8d058fd21e6dc760a4f94",
  "31a7ee4b63c511afaa7130052f073814fa7e94fba677954c1fb6446af5b596b0",
  "4b3c1235cd8927034420f6e841b95de2fed1f686a7a8cc7504c36aa339f820ce",
  "08867afb6f72cd4488d146a0024abb0740305e161964f3f765c181761aa0d964",
  "08c3ef584bbc9f6dd480a1ecd71773b9eedb7faf800ef208c7f5583650a4f3ff",
  "4a8db8ab904832fa3b45f80e57b7b26a8d111eba5d3b1084250f46417b2f81ea",
  "35e21f47986e561609484a7b99785a1c9babce997221730f0d0d47a505e4f7fe",
  "151e239fac18862b6dabc68a6c273d4eed6d64a3060f7cccf24d2c76e04ef0a0",
  "40467d25062f4dfa2d99e6a112c29c16e930a21b8c1ea36576d336a4a21423a0",
  "7536fabfcab0bfdec3ea3b4df23284973f3c341f83c74533baf4232940fe4e73",
  "49fae12b6b3c800f671e72ed161299d309a81a5d77305b53b9ae47c946300b73",
  "2d42771babd1310ef9ab87928d79a8afa29d6ac032db83906e3794c0e392622b",
  "4f1e159df8819130eb0cdbc0f2b741fe7551a7fc849231b79dedafc8bd980587",
  "3983eabee23c13aba838b8e47ff29d311bd722e817756741918d4dcc098fd10f",
  "60bfa89b994ac1ddec3ef34cd75998a37733e1ed8cc2b94a26c6da42254e0fd1",
  "740489156eaa51f39c912c5ac837922a014e5874635fafc1282e3a265a0e43ae",
  "02ae1ab2d3097429055d1be342d8c823a7fbd8c1d6afa712ebc7e98693c7109d",
  "5a3c7f297a7f1245f87132381f78651a8d34384216d19669e3d00ec3e0fbeefd",
  "4368758701bd066dce6449875689b92ee1ec9537ec533fbf85aabbe5c5a53914",
  "2cc3c09087a9c31a8acd300d8de85dc911d7c5436cbbf899504b59adbdcccd60",
  "1e84b37ea79a7e8791b3f25e02a9d4ce1f762cbf4a02b29324d56f9700b32d4d",
  "05f83a08feb40dd1cdbfaaa2cbc0640be818b4ef911c642f3b648b40344687cb",
  "259fc45c33a72fdcd38753289c578597ffc4e2be47b24948f69b90b7b927b7c4",
  "0a94523b4d24dd62812c03977874d04225aaca2529bf7d6a4d1ca3e96f62d3c4",
  "20ce1fb4f0df3069f1f959864b2389e3eff1f720ae121f84561b9489e4b75e90",
  "70d1e783118e4b0c5af62937537a118cb68365ba4e618915c97260702ff6d0f3",
  "1a65683f14de332305cfb620b8ee941f8ac2a9506906cbe356bbb3d5102e0490",
  "643be46ae26d8e0c060daf769b0511b8ba4dd6bffbfe67c789bb79138c269ce6",
  "5ae835b81bd6f94d96ebe1e555184efd902abe388ada34a3e2523a29c40efdb3",
  "7a6f6208866ad22b01c04e5f1498ef2951e9c3ad5299ae301c3ae57d7a4ce44c",
  "36154ee58e2d5069ff5c36900bac6f122cd1088d09900049e0c4c81dbc41d65d",
  "3cb473f3a10c302509c6c4556b633505fd081cd1f0f5a9e182a2d48e99c6f2a2",
  "241dc37822d752cf584c6ddf53e9e914ffcaf041de7143c2f3e347f23350d0ca",
  "6f3ed7866080a3b50b540c15515dfd60a5f93f47014574230170b6d9c7e37551",
  "2fda5c58001c3a071310ac0a0220f3b193ddcbe0f4ac82cfaa055cf9dc8ad91e",
  "6bee0a521efc11b2baf7d1c9d261034566573fde0978b7c84d7175782e8a3207",
  "2cf1652d2d8ce4c6d21dc0292ae29c749bbc526d7f64de227b4b4707fbb09097",
  "5860a35ba4338698c4aea2679332269f5ba08a67acced2a39c6d15255d91f52c",
  "765f0ccf7dc31b19de5cef65ace0354c5249fb2b2a5fc75d5e982f2bc3718e70",
  "3152a2b85bec47216e191d5915a1a7eabcb3a60dbfa746496573a8d1903c69e5",
  "7e94c7ce41153010e40e0bd189a5638ed7d69587bfc410d285235b03ef551e5e",
  "4e5076f2e53ccc1f0471c8ee12068873a018cdfda9a30db6cce958227582bb80",
  "45abce32515b573e4f2c85e174d66dc42c135111785ee954af3c1b534871239e",
  "2efcbf598bd96f1a687469c409b224a77c163a712bc2dd054e53950dad5ac9bd",
  "7bd1b2eef1177f78582fbf7eb4f79f3d9fd5133ab12b16dddad476aaf0433f27",
  "1046cab96202437ff9c29c04d03387cac2c8f19d322eeca3d7394753a66b6d90",
  "12116033b71fb2bd6705d1b1720b393b3ca68a7980d6dcd43b165a3414a2fe41",
  "21d811f4a51944885854bc9a3b4c0e42c5732cd11569dad849a05209b36928ff",
  "68b8a671a8ffa0dae72ca8da7a2c5f2d37856701bec8666a17beedd641a1541a",
  "02fa1a6e357c0122008a18c18dfce485f372da81ef3a01778b1adcba4ae761e4",
  "6d425748b3d4965b541ffa39daedacd137fa0d87f58f8e106c9480ba14bd9136",
  "3c22679213a917fab66c5d1e0c595a6cf05497c40765e31543b837a68b2cfafd",
  "45211d324ddd1c7da45aba0660cf8b57c03348dd30cd0260b16f0d97f97b888f",
  "0000deb741a7ebd2996612d38437744b08f0f20efa877251a1a37deb548c5661",
  "5dcd98cc37eea0e86f66f091acb042fd1d93f9401c2f2ec38ae217694bb9f144",
  "70d56c34ea32a9ed3279cdf2ac811aca239df6a1a320f365ff80344180765827",
  "644ffb5388b67ebeb4da6b447df4eeb65948a2607b0ffa03693f11d8189f4ab0",
  "74f4d86ae21f74233170cea6042c47c328684e150dd458a2ff6d2d18eca1fd2f"
]
Funded wallet with 101 blocks.

=== 2. CONFIRMATION STATUS ===
{
  "mine": {
    "trusted": 12675.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 825.00000000
  },
  "lastprocessedblock": {
    "hash": "74f4d86ae21f74233170cea6042c47c328684e150dd458a2ff6d2d18eca1fd2f",
    "height": 517
  }
}
[
  {
    "txid": "99c54425187d21e5169137fe3447157678712bbc4c4bdd9dbecca322ebd80ec1",
    "vout": 0,
    "address": "bcrt1qwz9yjuyxljyv9sa3g6k2xn7pjqll7sfsgk4g09",
    "label": "",
    "scriptPubKey": "0014708a497086fc88c2c3b146aca34fc1903fff4130",
    "amount": 12.50000000,
    "confirmations": 102,
    "spendable": true,
    "solvable": true,
    "desc": "wpkh([8a5a1a44/84h/1h/0h/0/12]03d16c3a4fd5ab9937e3c800e6cb09c38df1829027b0d00d09a08bafe234bfe64d)#0ynq8d2k",
    "parent_descs": [
      "wpkh(tpubD6NzVbkrYhZ4Wr7X5tiqwp4EdX5F6nQfEmaXjt8wey13PBREQQT9447RpyzoKRmk2rDHvagH785izSJfkxv2LqnPiFxgU8vDyB1aWtTcLKP/84h/1h/0h/0/*)#lwpjw49s"
    ],
    "safe": true
  },
  {
    "txid": "2da7f2c919da0d2008ff4eaeb1b6bb354db8caee3d257c4b7766e59f009eb198",
    "vout": 0,
    "address": "bcrt1qwz9yjuyxljyv9sa3g6k2xn7pjqll7sfsgk4g09",
    "label": "",
    "scriptPubKey": "0014708a497086fc88c2c3b146aca34fc1903fff4130",
    "amount": 12.50000000,
    "confirmations": 103,
    "spendable": true,
    "solvable": true,
    "desc": "wpkh([8a5a1a44/84h/1h/0h/0/12]03d16c3a4fd5ab9937e3c800e6cb09c38df1829027b0d00d09a08bafe234bfe64d)#0ynq8d2k",
    "parent_descs": [
      "wpkh(tpubD6NzVbkrYhZ4Wr7X5tiqwp4EdX5F6nQfEmaXjt8wey13PBREQQT9447RpyzoKRmk2rDHvagH785izSJfkxv2LqnPiFxgU8vDyB1aWtTcLKP/84h/1h/0h/0/*)#lwpjw49s"
    ],
    "safe": true
  }
]

=== 3. SPENDING ===
Spent 10 BTC. TXID: dedd6ca8fccca80f52ea78b2fd942ca56f710ce46d31a76a2bf466f493806f25
Mempool contains TX:
  "dedd6ca8fccca80f52ea78b2fd942ca56f710ce46d31a76a2bf466f493806f25"

=== Confirming Transaction ===
[
  "74ab7dfaa3c8d0fcbdcf8f2e61d9267d5b8be1b2b0f0aacebdfbfe8bc84d6dc4"
]

=== 4. DECODING TRANSACTION ===
{
  "txid": "dedd6ca8fccca80f52ea78b2fd942ca56f710ce46d31a76a2bf466f493806f25",
  "hash": "6c2d95ade2b0afef6a59ec57d1e4a91eaf986c69d39d372da2075c57b245d2ec",
  "version": 2,
  "size": 222,
  "vsize": 141,
  "weight": 561,
  "locktime": 517,
  "vin": [
    {
      "txid": "1d518d80a327df7e5d6953cce81c87253635fb1ff379ef5f5d1c169c9c9ff8c7",
      "vout": 0,
      "scriptSig": {
        "asm": "",
        "hex": ""
      },
      "txinwitness": [
        "3044022066374f6822fcacc60f06ddb25e076abe3123d94a55d7366b8db1a4ef95154700022059e2b55c140ec866fb620b2be7128c3522e0c16f27ee5827d1580bca3c069e3701",
        "02710975d093b6c435b0aade9f7743cde41f75b4b52babed1398bf47884a76752a"
      ],
      "prevout": {
        "generated": false,
        "height": 405,
        "value": 11.50001340,
        "scriptPubKey": {
          "asm": "0 1fe41c51291c472a85fca5f7e796d64e39b31f93",
          "desc": "addr(bcrt1qrljpc5ffr3rj4p0u5hm709kkfcumx8unsndnzs)#ad9swys7",
          "hex": "00141fe41c51291c472a85fca5f7e796d64e39b31f93",
          "address": "bcrt1qrljpc5ffr3rj4p0u5hm709kkfcumx8unsndnzs",
          "type": "witness_v0_keyhash"
        }
      },
      "sequence": 4294967293
    }
  ],
  "vout": [
    {
      "value": 10.00000000,
      "n": 0,
      "scriptPubKey": {
        "asm": "0 8f656847bba29c933d44bdcc5ef5649927839897",
        "desc": "addr(bcrt1q3ajks3am52wfx02yhhx9aatynync8xyh3qwqvj)#yr962km9",
        "hex": "00148f656847bba29c933d44bdcc5ef5649927839897",
        "address": "bcrt1q3ajks3am52wfx02yhhx9aatynync8xyh3qwqvj",
        "type": "witness_v0_keyhash"
      }
    },
    {
      "value": 1.49998520,
      "n": 1,
      "scriptPubKey": {
        "asm": "0 6bef807be1292291545be6d963ac687289624d8b",
        "desc": "addr(bcrt1qd0hcq7lp9y3fz4zmumvk8trgw2ykynvtkd7jyf)#x8ta5qms",
        "hex": "00146bef807be1292291545be6d963ac687289624d8b",
        "address": "bcrt1qd0hcq7lp9y3fz4zmumvk8trgw2ykynvtkd7jyf",
        "type": "witness_v0_keyhash"
      }
    }
  ],
  "fee": 0.00002820,
  "hex": "02000000000101c7f89f9c9c161c5d5fef79f31ffb353625871ce8cc53695d7edf27a3808d511d0000000000fdffffff0200ca9a3b000000001600148f656847bba29c933d44bdcc5ef5649927839897b8cbf008000000001600146bef807be1292291545be6d963ac687289624d8b02473044022066374f6822fcacc60f06ddb25e076abe3123d94a55d7366b8db1a4ef95154700022059e2b55c140ec866fb620b2be7128c3522e0c16f27ee5827d1580bca3c069e37012102710975d093b6c435b0aade9f7743cde41f75b4b52babed1398bf47884a76752a05020000",
  "blockhash": "74ab7dfaa3c8d0fcbdcf8f2e61d9267d5b8be1b2b0f0aacebdfbfe8bc84d6dc4",
  "confirmations": 1,
  "time": 1785758927,
  "blocktime": 1785758927
}

=== FINAL BALANCE ===
{
  "mine": {
    "trusted": 12687.49997180,
    "untrusted_pending": 0.00000000,
    "immature": 818.75002820
  },
  "lastprocessedblock": {
    "hash": "74ab7dfaa3c8d0fcbdcf8f2e61d9267d5b8be1b2b0f0aacebdfbfe8bc84d6dc4",
    "height": 518
  }
}

## Evidence references

TODO: Link screenshots or describe the attached evidence.

https://drive.google.com/drive/folders/1HvmkTC2bazkXgBELjgbLaaW8grJQgF9h?usp=sharing


## Explanation

TODO: Explain input combination, change, fees, and the privacy implication.

Every UTXO a wallet holds sits at its own address, and on its own, a UTXO reveals nothing about who else's coins belong to the same person. Before I spent anything, an outside observer watching the chain would only see three separate 0.4 BTC outputs sitting at three separate addresses — with no way to tell whether they belonged to one person or three different people.

The moment a transaction spends multiple UTXOs as inputs at the same time, that ambiguity disappears. A transaction is only valid if it's signed by the private keys controlling every one of its inputs — so consuming Alice's three 0.4 BTC UTXOs in a single spend is cryptographic proof, visible to anyone on the public chain, that the same person (or wallet) controlled all three simultaneously. My own transaction did exactly this: spend_input_count: 3 and funding_outpoints listing all three previously-separate outpoints together in one transaction is a permanent, public link between them. Anyone doing chain analysis can now say with certainty "these three addresses are commonly owned," something they couldn't have concluded before.

This is the core tension in UTXO-based coin selection: combining inputs is often the only way to reach a required payment amount — Alice genuinely needed all three 0.4 BTC UTXOs to cover a 1 BTC payment, since no single one of her outputs was large enough. But that mechanical necessity comes bundled with an involuntary privacy disclosure: the wallet had no way to spend 1 BTC without also revealing that these specific three UTXOs share an owner. The sender didn't choose to announce "these are all mine" — it was an unavoidable side effect of needing enough value in one place.

This is why privacy-conscious wallets often try to avoid unnecessary UTXO consolidation — for example, preferring to wait for one sufficiently large UTXO, or deliberately structuring earlier transactions to avoid needing to merge many small ones later — even though this can mean higher fees or waiting longer, purely to avoid revealing which coins actually belong together.