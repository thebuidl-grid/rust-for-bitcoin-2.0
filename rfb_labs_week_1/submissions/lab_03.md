# Lab 03 — Coinbase maturity

## Commands used

cargo build
cargo run --bin lab_03_usage

## Terminal output

miner_wallet: MyWallet
receiver_wallet: MyWallet2
miner_address: bcrt1q9x46sn23zs5cl8xdk22uuthw7cfrfymwk06lp5
Ok(WalletBalances { trusted: 0.0, untrusted_pending: 0.0, immature: 0.0 })
receiver_address: bcrt1qjxk5e992tj23tz92z2kgu52yg6vh9td6jx0zjg
attempt_payment: 
Err(Rpc("error code: -6\nerror message:\nInsufficient funds"))

result_coinbase: 
CoinbaseMaturityReport { 
    height_after_first_block: 2, 
    balance_after_first_block: WalletBalances { 
        trusted: 0.0, 
        untrusted_pending: 0.0, 
        immature: 50.0 
    }, 
    premature_spend_error: "error code: -6\nerror message:\nInsufficient funds", 
    final_height: 102, 
    final_balance: WalletBalances { 
        trusted: 50.0, 
        untrusted_pending: 0.0, 
        immature: 5000.0 
    } 
}

attempt_payment: 
Ok("55ee3dbaa7ca2789185f4fa23d05a5d292d76b1e3e0ff1c1cf12339de884a8ef")

## Evidence references

Description of the attached evidence:

Executes demonstrate_coinbase_maturity, which mines 1 additional block. Polar already mined 1 block when the node was created, so the block mined here becomes the second block. This is the first coinbase credit in the miner’s wallet. After that, get_balances is called, which shows the immature balance. Then 100 more blocks are mined, and the updated balance is reported. Finally, attempt_payment succeeds because more than 100 blocks have been mined since the original coinbase, so the reward is matured and spendable.


## Explanation

Explaination why the first coinbase reward becomes spendable at height 101:

Coins created by a coinbase transaction can’t be spent immediately. They must wait for COINBASE_MATURITY blocks first (100 on regtest by default).
A coin spends are allowed only after it has aged by at least COINBASE_MATURITY blocks. So the block that makes the coinbase from height 1 mature is height: 1+100=101. At height 101, that original coinbase has now completed 100 block intervals since it was mined, so its output is no longer considered immature and becomes part of trusted (spendable).