# Lab 09 — Multi-UTXO coin selection

## Commands used

cargo test --test lab_09
ALICE9_ADDRESS=$(docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=alice getnewaddress alice9)
RECEIVER9_ADDRESS=$(docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=receiver getnewaddress receiver9)
MINE9_ADDRESS=$(docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner getnewaddress mine9)

FUND_TXID_1=$(docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner sendtoaddress "$ALICE9_ADDRESS" 0.4)
FUND_TXID_2=$(docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner sendtoaddress "$ALICE9_ADDRESS" 0.4)
FUND_TXID_3=$(docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner sendtoaddress "$ALICE9_ADDRESS" 0.4)

docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie generatetoaddress 1 "$MINE9_ADDRESS"
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=alice listunspent

LAB9_SPEND_TXID=$(docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=alice sendtoaddress "$RECEIVER9_ADDRESS" 1.0)
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getrawtransaction "$LAB9_SPEND_TXID" true
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=receiver gettransaction "$LAB9_SPEND_TXID"

## Terminal output

- Generated addresses:
	- Alice: `bcrt1q5v93tpqjj0jmp9a939rn7qlhf0fuxe9eecd7aa`
	- Receiver: `bcrt1qegdz03g3w6745axqkmajvxk5juxgftx0wqr077`
	- Miner: `bcrt1qamlymstwz2k6499ppzzgn2mkpmpfrmf5axwf0g`
- Funding TXIDs:
	- `4b0da82146c9ed15713ce70d9756c7570a1ea53b17c5c6b5af4cd0c21638796b`
	- `b7d1e36a3de9a2e53d751a53e137afea0afbfc3513d93cebaeb89a505ddf410a`
	- `89a1682548d2582e104bde4946d52cc8af62b41a6327063e1a3cd8b165b42f60`
- After mining 1 block, `listunspent` for Alice shows three confirmed UTXOs, each `0.40000000` BTC.
- Spend transaction:
	- `LAB9_SPEND_TXID = 05b6103c06f044f0a54ee00ce8b749e478dfa116c4a7442773896be59fa8b532`
	- `vin` count: `3` (consumes all three Alice funding UTXOs)
	- `vout` count: `2`
	- receiver output (`n=0`): `1.00000000` to `bcrt1qegdz03g3w6745axqkmajvxk5juxgftx0wqr077`
	- change output (`n=1`): `0.19994480` to `bcrt1qy6pcvc04cek7rea479ayy6qaswh7hcy2hnqx20`
	- size/vsize: `518` / `276`
- Receiver wallet view for the same tx:
	- amount: `1.00000000`
	- confirmations: `0`
	- trusted: `false`

## Evidence references

- `submissions/evidence/lab9.png` (terminal view of wallet setup, funding, and confirmed Alice UTXOs)
- `submissions/evidence/lab9-02.png` (terminal view of spend transaction decode and receiver wallet view)

## Explanation

Alice had three separate confirmed UTXOs of `0.4 BTC` each, so the wallet combined them to fund a `1.0 BTC` payment. The decoded transaction proves this with three inputs (`vin` length = 3).

Total input value is `1.2 BTC` and total output value is `1.19994480 BTC` (`1.00000000 + 0.19994480`). The implied miner fee is the difference:

`fee = 1.20000000 - 1.19994480 = 0.00005520 BTC`.

Privacy implication: combining multiple UTXOs in one transaction links those UTXOs to the same spender. An observer can cluster the three Alice inputs together, which reduces address/UTXO-level privacy.
