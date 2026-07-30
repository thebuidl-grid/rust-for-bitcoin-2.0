# Lab 04 — UTXOs and outpoints

## Commands used

cargo test --test lab_04
cargo run --example lab04_check
docker exec polar-n3-backend1 bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=miner getbalance


## Terminal output

miner UTXOs: [
    Utxo {
        txid: "dda96d926694608d4699a9858ec6ac324f9633a7a3a64562aa76389285ca1be3",
        vout: 0,
        address: Some(
            "bcrt1qq6jewkpw6yv97xpxkt8yf2j33p68fhe7kn4sfc",
        ),
        script_pub_key: "001406a597582ed1185f1826b2ce44aa51887474df3e",
        amount: 50.0,
        confirmations: 101,
        spendable: true,
    },
]
selected spendable UTXO: Some(
    Utxo {
        txid: "dda96d926694608d4699a9858ec6ac324f9633a7a3a64562aa76389285ca1be3",
        vout: 0,
        address: Some(
            "bcrt1qq6jewkpw6yv97xpxkt8yf2j33p68fhe7kn4sfc",
        ),
        script_pub_key: "001406a597582ed1185f1826b2ce44aa51887474df3e",
        amount: 50.0,
        confirmations: 101,
        spendable: true,
    },
)
outpoint: OutPoint { txid: "dda96d926694608d4699a9858ec6ac324f9633a7a3a64562aa76389285ca1be3", vout: 0 }
sum of spendable UTXOs: 50


## Evidence references

https://drive.google.com/drive/folders/1mP1ycuASg9SOfhFiHK00MdBMmprZZjQp?usp=drive_link

## Explanation
A traditional bank balance is a single mutable number stored in a database row — the bank decrements it directly when you spend. Bitcoin doesn't work that way: there's no stored "balance" field anywhere. What Bitcoin Core actually tracks is a set of discrete, individually-addressable coins — UTXOs (unspent transaction outputs) — each identified by its own unique txid:vout coordinate. A "balance" is something the wallet computes on demand by scanning every UTXO it controls, filtering to the ones it considers spendable, and summing their amounts.

I proved this directly: sum_spendable_utxos in my own Rust code independently summed the miner wallet's UTXO set to 50, and bitcoin-cli getbalance — Bitcoin Core's own built-in calculation — returned 50.00000000. Both numbers came from the same underlying source of truth (the UTXO set), computed two different ways, and they matched exactly. That's only possible because the "balance" isn't a stored value being read twice — it's derived fresh from the coins each time.

This distinction matters in practice: a UTXO is atomic. You can't partially spend a 50 BTC output the way you'd debit part of a bank balance — spending it consumes the entire 50 BTC output, and if you only want to send less than that, the transaction has to create a brand-new "change" output sending the remainder back to yourself. A wallet balance is really just an aggregate view over however many of these atomic, indivisible coins happen to be sitting in your wallet at that moment — not a single entry that goes up and down.
