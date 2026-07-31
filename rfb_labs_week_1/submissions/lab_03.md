# Lab 03 — Coinbase maturity

## Commands used

```bash
cargo run -- lab03
```

`demonstrate_coinbase_maturity` runs this sequence:

```bash
bitcoin-cli ... generatetoaddress 1 bcrt1qfsw0fvcdjruj7d746sxqy0nnnpptcvsyslhx0q
bitcoin-cli ... getblockcount
bitcoin-cli ... -rpcwallet=miner getbalances
bitcoin-cli ... -rpcwallet=miner sendtoaddress bcrt1qga5wdzs456gvrk7kzh07lxxm5lxjarslkxm3m4 1
bitcoin-cli ... generatetoaddress 100 bcrt1qfsw0fvcdjruj7d746sxqy0nnnpptcvsyslhx0q
bitcoin-cli ... getblockcount
bitcoin-cli ... -rpcwallet=miner getbalances
```

The `sendtoaddress` is expected to fail. My implementation captures the error message
into the report rather than swallowing it, and it treats an unexpected *success* as a
failure of the lab, since a payment that goes through proves nothing about maturity.

## Terminal output

After mining the first block to the miner address:

```text
$ bitcoin-cli ... generatetoaddress 1 bcrt1qfsw0f...
  [ "640dd92bfdaf585098e6e12778a1ab4ea1ade00349d307808d17da97463baecb" ]
$ bitcoin-cli ... getblockcount
  2
$ bitcoin-cli ... -rpcwallet=miner getbalances
  { "mine": { "trusted": 0.00000000, "untrusted_pending": 0.00000000, "immature": 50.00000000 } }
```

The premature spend, preserved verbatim:

```text
$ bitcoin-cli ... -rpcwallet=miner sendtoaddress bcrt1qga5wdzs...m3m4 1
  !! Bitcoin Core RPC failed: error code: -6
error message:
Insufficient funds
```

After mining 100 more:

```text
$ bitcoin-cli ... getblockcount
  102
$ bitcoin-cli ... -rpcwallet=miner getbalances
  { "mine": { "trusted": 50.00000000, "untrusted_pending": 0.00000000, "immature": 5000.00000000 } }
```

Final report:

```json
{
  "height_after_first_block": 2,
  "balance_after_first_block": { "trusted": 0.0, "untrusted_pending": 0.0, "immature": 50.0 },
  "premature_spend_error": "error code: -6\nerror message:\nInsufficient funds",
  "final_height": 102,
  "final_balance": { "trusted": 50.0, "untrusted_pending": 0.0, "immature": 5000.0 }
}
```

## Evidence references

Full run log at `evidence/week1-labs-01-09.log`, lines 162-322. The 100 block hashes
returned by the bulk `generatetoaddress` are in that log at lines 189-291 and are not
reproduced here.

## Explanation

A coinbase output — the block reward plus fees paid to whoever mined the block — cannot
be spent until 100 further blocks are built on top of the block that created it. That is
`COINBASE_MATURITY = 100` in the consensus rules. Before then the wallet reports the
value under `immature` rather than `trusted`, and it will not select those coins for
spending.

My run shows this cleanly. Right after mining one block the wallet held 50 BTC immature
and 0 trusted, so asking it to send 1 BTC failed with `Insufficient funds` (error `-6`).
The wallet was not confused about owning the coins; it simply refused to count coins that
consensus would not yet let it spend. After 100 more blocks that first reward had 100
confirmations and moved into `trusted: 50.0`, while the 100 newer rewards took its place
as `immature: 5000.0` — exactly 100 × 50 BTC still waiting.

The reason for the rule is reorganizations. If a competing branch replaces the block that
created a coinbase, that coinbase ceases to exist — unlike an ordinary transaction, it
cannot simply be re-mined into the new chain, because it was minted by that specific
block. Anything spending it would become invalid too, and the damage would cascade
through every downstream transaction. Requiring 100 blocks of depth means that by the
time the reward is spendable, a reorg deep enough to destroy it is impractical.

On the conventional 101: on a chain starting at height 0, mining 101 blocks puts the
first reward (block 1) at exactly 100 confirmations, which is the minimum to spend it,
and this is why the lab describes height 101. My chain started at height 1 because Polar
mines a block when it creates the network, so my equivalent milestone is 102 — the first
*miner* reward was mined at height 2 and became spendable at height 102. The 100-block
rule is unchanged; only the starting offset differs.
