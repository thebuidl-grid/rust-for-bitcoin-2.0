# Lab 10 — Competing branches and reorganization

## Commands used

cargo test --test lab_10
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getchaintips
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getbestblockhash
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getblockheader "$(docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getbestblockhash)"
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getpeerinfo
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie disconnectnode "127.0.0.1:18444"
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie addnode "127.0.0.1:18444" onetry
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getpeerinfo
docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getchaintips

## Terminal output

- Initial chain tip (`getchaintips`):
	- `height: 211`
	- `hash: 59b9c95c1e8185b9c51a083e85f3bb115cd3625be97986f962e2d8ad9f70d598`
	- `branchlen: 0`, `status: active`
- Best block hash (`getbestblockhash`):
	- `59b9c95c1e8185b9c51a083e85f3bb115cd3625be97986f962e2d8ad9f70d598`
- Best-tip header (`getblockheader`):
	- `height: 211`
	- `hash: 59b9c95c1e8185b9c51a083e85f3bb115cd3625be97986f962e2d8ad9f70d598`
	- `previousblockhash: 54c937a986cb2b5fee2d88aa183d18fc7149450683b4fbeff1498911f58f9cb5`
	- `chainwork: 00000000000000000000000000000000000000000000000000000000000001a8`
	- `merkleroot: 34e133911c7fe4d86afd53d7decc2772c5744db0efee8890033277df62c96845`
- Peer operations:
	- Initial `getpeerinfo`: `[]` (no connected peers)
	- `disconnectnode "127.0.0.1:18444"` returned:
		- `error code: -29`
		- `error message: Node not found in connected nodes`
	- `addnode "127.0.0.1:18444" onetry` succeeded; follow-up `getpeerinfo` showed two entries for the loopback session:
		- outbound manual peer `127.0.0.1:18444`
		- inbound peer `127.0.0.1:37316`
- Final chain tip (`getchaintips`) remained:
	- `height: 211`, `hash: 59b9c95c1e8185b9c51a083e85f3bb115cd3625be97986f962e2d8ad9f70d598`, `status: active`

## Evidence references

- `screenshots/lab10.png` (terminal view of chain-tip and best-header inspection)
- `screenshots/lab10-02.png` (terminal view of peer/disconnect/reconnect commands and outputs)
- `screenshots/lab10-03.png` (terminal view confirming final chain tip status)

## Explanation

This run demonstrates tip inspection and peer-link control used during reorg experiments. The active tip is identified by hash, height, and chainwork. `chainwork` is the key security metric because nodes follow the valid chain with the most cumulative proof of work.

The `disconnectnode` error (`-29`) is expected in this trace because there were no connected peers at the time of the disconnect call (`getpeerinfo` returned an empty array first). After `addnode ... onetry`, the node established a loopback connection and `getpeerinfo` confirmed both outbound and inbound entries.

No competing branch was present at the end of this run (`getchaintips` showed only one active tip with `branchlen: 0`). In a true two-branch scenario, the stale branch would appear as a non-active tip; once a stronger branch accumulates more work, the node reorganizes to that branch and treats blocks from the weaker branch as stale.
