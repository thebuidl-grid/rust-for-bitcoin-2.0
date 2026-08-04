# Lab 08 — Block security

## Commands used

```
cargo test --test lab_08
bitcoin-cli -regtest getblockheader "<block-hash>"
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "<txid>"
bitcoin-cli -regtest generatetoaddress 5 "<miner-address>"
bitcoin-cli -regtest -rpcwallet=receiver gettransaction "<txid>"
```

*RPCs are the ones issued by `get_block_header`, `get_confirmations`, `mine_additional_blocks`, and `build_security_report` in `src/labs/lab08_security.rs`, verified against the mocked RPC client in `tests/lab_08.rs`. Run the `bitcoin-cli` lines against your live Polar regtest node to capture the terminal output below.*

## Terminal output

Captured against the live regtest node, continuing from Lab 07 (block `677f6328...` at height 102):

```
$ bitcoin-cli -regtest getblockheader "677f6328af7710f6b8fce2cac04a6c23eb2fe8dc2dacb73fd6896292992857c8"
{
  "hash": "677f6328af7710f6b8fce2cac04a6c23eb2fe8dc2dacb73fd6896292992857c8",
  "confirmations": 1,
  "height": 102,
  "merkleroot": "824555d5ba29d9c8aa68c64bd1e6c79d1279ee89bc8ba4ab1b7b661dce24c0f9",
  "nonce": 0,
  "bits": "207fffff",
  "difficulty": 4.656542373906925e-10,
  "chainwork": "00000000000000000000000000000000000000000000000000000000000000ce",
  "previousblockhash": "20774b91b25a63e16a078d32fb2306c9461ff0bd51e22f673c3b9c4d96db5f7d"
}

$ bitcoin-cli -regtest -rpcwallet=receiver gettransaction "7db84ed92ac38e4c6f01412011f9f97098cedd8a259c646c9cd1f192a9ff84c2"
{
  "amount": 1.00000000,
  "confirmations": 1,
  "blockhash": "677f6328af7710f6b8fce2cac04a6c23eb2fe8dc2dacb73fd6896292992857c8",
  ...
}

$ bitcoin-cli -regtest generatetoaddress 5 "bcrt1qtdwur5a220ta9f0lndtdqf45f0kmaplt3xea5l"
[
  "440c957580d889716dfc9bdb776b1ac9493716e6cffa7c09f5a5831ceb779464",
  "6f75c44963c73c2e683db0a1813bd8731a56d5a12027ae4f09f63848ab8bd214",
  "1c662b66287aafbdd96411c893acdb4aa80e6e24e9bcb186f8e216b5af1a8595",
  "56f299351cdfc69c3fdea0136e5f02a52e89322c45ce7a7d9d7f8367281a89e9",
  "5660455531e12901383df0061c96e556e0f1767ec8eba3056dfdccbd17a55588"
]

$ bitcoin-cli -regtest -rpcwallet=receiver gettransaction "7db84ed92ac38e4c6f01412011f9f97098cedd8a259c646c9cd1f192a9ff84c2"
{
  "amount": 1.00000000,
  "confirmations": 6,
  "blockhash": "677f6328af7710f6b8fce2cac04a6c23eb2fe8dc2dacb73fd6896292992857c8",
  ...
}
```

The header links to its parent via `previousblockhash` and commits to its transactions via `merkleroot`; `chainwork` records accumulated proof of work up to and including this block. Before mining 5 more blocks the transaction had `confirmations: 1`; after, `confirmations: 6` — one for the confirming block itself plus five stacked on top, exactly matching `1 + 5 = 6`.

## Evidence references

Evidence is the live terminal output above, captured directly via `docker exec bitcoind-lab-a bitcoin-cli ...` against a real regtest node (not a screenshot — this session ran headlessly, no Polar GUI was open).

## Explanation

**Hash links:** every block header stores `previousblockhash` — the hash of the block before it. Since a block's own hash is computed *from* its header (which includes that previous hash), you can't change any earlier block without changing its hash, which breaks every `previousblockhash` reference after it. This chains all blocks together into an unbroken, tamper-evident sequence — the reason tampering with block 102 here (previous hash `20774b91...`) would be immediately detectable by every node that already has it.

**Merkle root:** `merkleroot` is a single hash that summarizes every transaction in the block, built by repeatedly hashing pairs of transactions together up to one root. It lets you prove a specific transaction is in a block using only a short path of hashes, without needing the whole block — and if even one transaction were altered, the merkle root would change, so it acts as a tamper-evident fingerprint of the block's contents.

**Proof of work:** to be accepted, a block's header hash must be numerically below a target (encoded in `bits`); miners find this by trying different `nonce` values until one works. `chainwork` (`...ce` here) is the running total of expected work spent producing this block and every block before it — it's how nodes objectively compare which of two competing chains represents more real computational effort.

**Confirmation depth:** simply how many blocks (including the confirming one) sit on top of a transaction. It's a proxy for security: each additional block makes reversing that transaction require redoing that much more proof of work. Going from `confirmations: 1` to `confirmations: 6` after mining 5 more blocks shows this directly — the transaction is exactly as buried as the number of blocks mined since it was confirmed.
