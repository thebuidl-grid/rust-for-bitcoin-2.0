# Lab 03 — Coinbase maturity

## Commands used

cargo test --test lab_03
cargo run --example lab03_check


## Terminal output

`CoinbaseMaturityReport {
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
}`


## Evidence references

https://drive.google.com/drive/folders/1mP1ycuASg9SOfhFiHK00MdBMmprZZjQp?usp=drive_link

## Explanation


A coinbase transaction is special: it's not paid by anyone, it's created out of nothing as the block reward. That makes it uniquely vulnerable to chain reorganizations. If a node lets you spend a coinbase reward the instant it was mined, and that block later got orphaned by a longer competing chain , the coins you spent would never have existed on the chain that actually won — but anything you bought with them, or any further transactions built on top of that spend, would already be out in the world.
Bitcoin Core prevents this by refusing to let a wallet spend a coinbase output until it's buried under COINBASE_MATURITY = 100 additional blocks. By that depth, reorganizing the chain deep enough to undo it would require an attacker to out-mine 100 blocks of accumulated proof-of-work — practically infeasible — so the reward can be treated as final.

My own run demonstrated this directly: right after mining the first block, get_balances reported trusted: 0.0 / immature: 50.0 — the 50 BTC reward existed but Bitcoin Core wouldn't count it as spendable. Attempting to send 1 BTC to the receiver at that point failed with a real RPC error (-6 Insufficient funds), even though the wallet's total balance already showed 50 BTC — proof the immaturity restriction is enforced by the node, not just a display convention.

Why the lab mines 101 blocks on a fresh chain

The maturity rule counts confirmations relative to the block containing the coinbase, not the mining event itself. On a brand-new chain (height 0), mining one block creates the reward at height 1. For that reward to have 100 confirmations, the chain needs to reach height 101 — the original block plus 100 more stacked on top of it. That's why the lab's conventional recipe is "mine 1, then mine 100 more": it's not two arbitrary numbers, it's "create the reward" followed by exactly the depth required to mature it.

My chain wasn't perfectly fresh — it already had one block from Polar's initial setup, so my own heights were offset by one: the reward was created at height 2, not height 1, and matured once the chain reached height 102, not 101.

