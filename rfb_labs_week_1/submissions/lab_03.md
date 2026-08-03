# Lab 03 — Coinbase maturity

## Commands used
# Get new address from mywallet1
ADDR=$(docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 getnewaddress)

# Mine 101 blocks to that address
docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass generatetoaddress 101 "$ADDR"   
TODO: Record mining, balance inspection, and premature-spend commands.

docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=mywallet1 getbalances   

## Terminal output

{
  "mine": {
    "trusted": 8800.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 2450.00000000
  },
  "lastprocessedblock": {
    "hash": "27b9834fd51427cd5631599c828df1f565b831fb0f82d83e86cc0ecd9a1979c6",
    "height": 303
  }
}
TODO: Show balances at heights 1 and 101 plus the failed premature spend.

## Evidence references

https://drive.google.com/drive/folders/1HvmkTC2bazkXgBELjgbLaaW8grJQgF9h?usp=sharing


TODO: Link screenshots or describe the attached evidence.

## Explanation

TODO: Explain why the first coinbase reward becomes spendable at height 101.

A coinbase transaction is special: it's not paid by anyone, it's created out of nothing as the block reward. That makes it uniquely vulnerable to chain reorganizations. If a node lets you spend a coinbase reward the instant it was mined, and that block later got orphaned by a longer competing chain , the coins you spent would never have existed on the chain that actually won — but anything you bought with them, or any further transactions built on top of that spend, would already be out in the world.
Bitcoin Core prevents this by refusing to let a wallet spend a coinbase output until it's buried under COINBASE_MATURITY = 100 additional blocks. By that depth, reorganizing the chain deep enough to undo it would require an attacker to out-mine 100 blocks of accumulated proof-of-work — practically infeasible — so the reward can be treated as final.

My own run demonstrated this directly: right after mining the first block, get_balances reported trusted: 0.0 / immature: 50.0 — the 50 BTC reward existed but Bitcoin Core wouldn't count it as spendable. Attempting to send 1 BTC to the receiver at that point failed with a real RPC error (-6 Insufficient funds), even though the wallet's total balance already showed 50 BTC — proof the immaturity restriction is enforced by the node, not just a display convention.

Why the lab mines 101 blocks on a fresh chain

The maturity rule counts confirmations relative to the block containing the coinbase, not the mining event itself. On a brand-new chain (height 0), mining one block creates the reward at height 1. For that reward to have 100 confirmations, the chain needs to reach height 101 — the original block plus 100 more stacked on top of it. That's why the lab's conventional recipe is "mine 1, then mine 100 more": it's not two arbitrary numbers, it's "create the reward" followed by exactly the depth required to mature it.
