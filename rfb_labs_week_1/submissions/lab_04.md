# Lab 04 — UTXOs and outpoints

## Commands used

cargo build
cargo run --bin lab_04_usage

fn lab04() -> Result<(), Box<dyn std::error::Error>> {
    let miner_wallet = "MyWallet";
    let receiver_wallet = "MyWallet2";
    let client = ProcessRpc::default().with_base_args(["-regtest"]);
    let _ = create_wallet(&client, miner_wallet);
    let _ = create_wallet(&client, receiver_wallet);
    println!("miner_wallet: {}", miner_wallet);
    println!("receiver_wallet: {}", receiver_wallet);

    let miner_address = get_new_address(&client, miner_wallet, "miner_address")?;
    println!("miner_address: {}", miner_address);
    labs::lab03_maturity::mine_blocks(&client, &miner_address, 101)?;
    println!("101 blocks mined");

    let receiver_address = get_new_address(&client, receiver_wallet, "receiver_address")?;
    println!("receiver_address: {}", receiver_address);
    let result_attempt_payment =
        labs::lab03_maturity::attempt_payment(&client, miner_wallet, &receiver_address, 1.0);
    println!("attempt_payment 1: ");
    println!("{result_attempt_payment:?}");
 
    labs::lab03_maturity::mine_blocks(&client, &miner_address, 1)?;
    println!("1 block mined");

    let result_attempt_payment2 =
        labs::lab03_maturity::attempt_payment(&client, miner_wallet, &receiver_address, 2.0);
    println!("attempt_payment 2: ");
    println!("{result_attempt_payment2:?}");

    labs::lab03_maturity::mine_blocks(&client, &miner_address, 1)?;
    println!("1 block mined");

    let result_unspent = list_unspent(&client, receiver_wallet)?;
    println!("receiver unspent: {result_unspent:?}");


    Ok(())
}


## Terminal output

Includes txid, vout, amount, confirmations, script, and spendable state.

miner_wallet: MyWallet
receiver_wallet: MyWallet2
miner_address: bcrt1qqmfr8jj75p7lq26rluv0zalhpfvy7kykha0699
101 blocks mined
receiver_address: bcrt1qr4k0v58r8khd5faa7c9ecsfjhucjd2kpp582q5
attempt_payment 1: 
Ok("6f134a708257f83687bdb73066cb30669b27b98c309213d3bd8123eecc35f18b")
1 block mined
attempt_payment 2: 
Ok("7e302aa46827efebb9513dbca15999fe4a951725ccf34afdcb13953fdf619518")
1 block mined
receiver unspent: [
    Utxo { 
        txid: "7e302aa46827efebb9513dbca15999fe4a951725ccf34afdcb13953fdf619518", 
        vout: 0, 
        address: Some("bcrt1qr4k0v58r8khd5faa7c9ecsfjhucjd2kpp582q5"), 
        script_pub_key: "00141d6cf650e33daeda27bdf60b9c4132bf3126aac1", 
        amount: 2.0, 
        confirmations: 1, 
        spendable: true 
    }, Utxo { 
        txid: "6f134a708257f83687bdb73066cb30669b27b98c309213d3bd8123eecc35f18b", 
        vout: 1, 
        address: Some("bcrt1qr4k0v58r8khd5faa7c9ecsfjhucjd2kpp582q5"), 
        script_pub_key: "00141d6cf650e33daeda27bdf60b9c4132bf3126aac1", 
        amount: 1.0, 
        confirmations: 2, 
        spendable: true 
    }
]


## Evidence references

1. **Creates RPC client and wallets**
   - Instantiates an RPC client configured with `-regtest`.
   - Creates:
     - `miner_wallet` (sender / miner)
     - `receiver_wallet` (recipient)

2. **Generates wallet addresses**
   - Requests a new mining address from `miner_wallet` (`miner_address`).
   - Requests a new receiving address from `receiver_wallet` (`receiver_address`).

3. **Mines blocks to reach coinbase maturity**
   - Mines **101 blocks** to `miner_address`:
     - This is done to make the earliest coinbase reward become **spendable** after the coinbase maturity window.

4. **Attempts payments**
   - Attempts to send **1.0 BTC** from `miner_wallet` to `receiver_address` and prints the result.
   - Mines **1 more block**.
   - Attempts to send **2.0 BTC** and prints the result.
   - Mines **1 more block** again.

5. **Lists receiver unspent outputs**
   - Calls `list_unspent(&client, receiver_wallet)` to print the receiver wallet’s tracked UTXOs.


## Explanation

An **OutPoint** uniquely identifies a specific transaction output. It is the pair:
- **`txid`**: the hash of the transaction that created the output
- **`vout`**: the index of that output inside the transaction (0, 1, 2, ...)

So an OutPoint is effectively **(txid, vout)**. When you spend coins, your transaction’s inputs (`vin`) reference the exact OutPoint(s) being spent.

A **UTXO** (“Unspent Transaction Output”) is a transaction output that:
- was created by some past transaction, and
- has **not** been spent by any later transaction.

Each UTXO corresponds to exactly one OutPoint, and that OutPoint tells you *which* output is still available to spend.

A wallet controls certain scripts/keys (e.g., belonging to its addresses). The wallet:
1. finds the UTXOs it controls that are still unspent,
2. reads each UTXO’s **`amount`**, and
3. reports balances as the **sum** of those amounts

The wallet’s balance is the sum of the amounts in the wallet’s tracked, unspent UTXOs. Spent outputs are no longer included.
