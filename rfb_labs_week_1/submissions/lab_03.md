# Lab 03 — Coinbase maturity

## Commands used

Rust: `cargo run --example run` (calls `demonstrate_coinbase_maturity`, which internally runs):
- `generatetoaddress 1 <miner_address>` — mine one block
- `getblockcount` — read height after first block
- `getbalances` (wallet: miner) — read balance after first block
- `sendtoaddress <receiver_address> 1` (wallet: miner) — attempt premature spend
- `generatetoaddress 100 <miner_address>` — mine 100 more blocks
- `getblockcount` — read final height
- `getbalances` (wallet: miner) — read final balance

## Terminal output

=== Lab 03: coinbase maturity ===
CoinbaseMaturityReport {
height_after_first_block: 2,
balance_after_first_block: WalletBalances {
trusted: 0.0,
untrusted_pending: 0.0,
immature: 50.0,
},
premature_spend_error: "error code: -6\nerror message:\nInsufficient funds",
final_height: 102,
final_balance: WalletBalances {
trusted: 50.0,
untrusted_pending: 0.0,
immature: 5000.0,
},
}

## Evidence references

Screenshot: `evidence/lab03.png`

## Explanation

Bitcoin enforces a 100-block maturity rule on coinbase rewards (the newly-minted coins a miner receives for mining a block). A coinbase output cannot be spent until 100 additional blocks have been mined on top of it — this exists to protect against a chain reorganization erasing a block after its reward has already been spent elsewhere, which would let someone spend coins that turn out to have never really existed.

My chain wasn't at height 0 when this lab ran (I'd already mined a block earlier in Lab 01), so my numbers are offset from the textbook "spendable at height 101" example — but the underlying rule is identical. Right after mining one block, I was at height 2 with the full 50 BTC reward sitting entirely in `immature` — none of it in `trusted`. Trying to spend it immediately failed with "Insufficient funds," even though `listwallets`/the wallet clearly showed 50 BTC existed — Bitcoin Core simply refuses to count immature coins as spendable balance at all.

After mining 100 more blocks (reaching height 102 — exactly 100 blocks past the block that paid the reward), that original 50 BTC finally shows up in `trusted` and became spendable. The `immature` balance at that point (5000.0) is the reward from the 100 *new* blocks I just mined to get there — each of those is now itself waiting out its own 100-block clock before it matures in turn.