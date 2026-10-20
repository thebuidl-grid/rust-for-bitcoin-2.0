# Lab 01 — Regtest network inspection

## Commands used
- `bitcoin-cli -regtest getblockchaininfo` — backs `get_chain` (reads the `chain` field)
- `bitcoin-cli -regtest getblockcount` — backs `get_block_height`
- `bitcoin-cli -regtest getbestblockhash` — backs `get_best_block_hash`
- These three are composed together by `inspect_network`, which additionally
  rejects any chain that isn't `"regtest"` before returning a `NetworkSnapshot`.

## Terminal output
$ bitcoin-cli -regtest getblockchaininfo
{
"chain": "regtest",
"blocks": 587,
"headers": 587,
"bestblockhash": "227e813054b890576163bbf4aadc669e594666e03fe2585e4f081878c8143e26",
"difficulty": 4.656542373906925e-10,
"time": 1785241778,
"mediantime": 1785241777,
"verificationprogress": 1,
"initialblockdownload": false,
"chainwork": "0000000000000000000000000000000000000000000000000000000000000498",
"size_on_disk": 176213,
"pruned": false,
"warnings": ""
}

$ bitcoin-cli -regtest getblockcount
587

$ bitcoin-cli -regtest getbestblockhash
227e813054b890576163bbf4aadc669e594666e03fe2585e4f081878c8143e26

## Evidence references
Captured directly from a local Bitcoin Core node running in regtest mode
(`~/.bitcoin`, `bitcoin.conf` with `regtest=1`, `txindex=1`). Raw terminal
output pasted above was copied verbatim from the shell.

## Explanation (co-authored by Claude)

Bitcoin Core is the reference implementation of the Bitcoin protocol, the actual node software that validates blocks, relays transactions, and exposes an RPC interface (bitcoin-cli) for control. Regtest ("regression test mode") is one of Bitcoin Core's private network modes, alongside mainnet and testnet. Unlike mainnet, where new blocks require real proof-of-work at real difficulty, regtest has near-zero difficulty and blocks can be mined instantly on demand via generatetoaddress making it ideal for fast, repeatable local development and testing without spending real money or waiting for real block times. Docker is a containerization tool that packages an application (like bitcoind) with all its dependencies into an isolated, reproducible environment; Polar is a GUI built on top of Docker that spins up and manages multiple such Bitcoin Core (and Lightning) containers with a few clicks, which is convenient for multi-node setups but not required Bitcoin Core can be run directly on bare metal, as done here, with the same RPC surface.
