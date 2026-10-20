# Lab 09 — Multi-UTXO coin selection

## Commands used

`cargo test --test lab_09
cargo run --example lab09_check
`

## Terminal output

`alice address: bcrt1qeg8wcwhrdy689dm4zcuenq5gzqslvvulcz50zq
funding txids: ["0ba3db28ff339ee2d0ed4023e3d58e92869f50b90f8ac3e8f40f7e4bf8fd6958", "570e3be77bcd4e9969f2c848f84348a0934ef89db579c238efc08a60ee77a83a", "ea43e849fef97fa3e9f3878232b924a1934087e98739038a40058ba0d47a4301"]
alice's confirmed UTXOs: [
    Utxo {
        txid: "ea43e849fef97fa3e9f3878232b924a1934087e98739038a40058ba0d47a4301",
        vout: 1,
        address: Some(
            "bcrt1qeg8wcwhrdy689dm4zcuenq5gzqslvvulcz50zq",
        ),
        script_pub_key: "0014ca0eec3ae3693472b77516399982881021f6339f",
        amount: 0.4,
        confirmations: 1,
        spendable: true,
    },
    Utxo {
        txid: "570e3be77bcd4e9969f2c848f84348a0934ef89db579c238efc08a60ee77a83a",
        vout: 1,
        address: Some(
            "bcrt1qeg8wcwhrdy689dm4zcuenq5gzqslvvulcz50zq",
        ),
        script_pub_key: "0014ca0eec3ae3693472b77516399982881021f6339f",
        amount: 0.4,
        confirmations: 1,
        spendable: true,
    },
    Utxo {
        txid: "0ba3db28ff339ee2d0ed4023e3d58e92869f50b90f8ac3e8f40f7e4bf8fd6958",
        vout: 0,
        address: Some(
            "bcrt1qeg8wcwhrdy689dm4zcuenq5gzqslvvulcz50zq",
        ),
        script_pub_key: "0014ca0eec3ae3693472b77516399982881021f6339f",
        amount: 0.4,
        confirmations: 1,
        spendable: true,
    },
]
new receiver address: bcrt1q4kl02lpztynp25lnc50rljr5c9dpf8x8mkxph6
MultiUtxoAudit {
    funding_outpoints: [
        OutPoint {
            txid: "ea43e849fef97fa3e9f3878232b924a1934087e98739038a40058ba0d47a4301",
            vout: 1,
        },
        OutPoint {
            txid: "570e3be77bcd4e9969f2c848f84348a0934ef89db579c238efc08a60ee77a83a",
            vout: 1,
        },
        OutPoint {
            txid: "0ba3db28ff339ee2d0ed4023e3d58e92869f50b90f8ac3e8f40f7e4bf8fd6958",
            vout: 0,
        },
    ],
    spend_txid: "a612d023d9a5e962817213aa90ec518bcd452cf90b6ed0c484650af02552f32a",
    spend_input_count: 3,
    payment_and_change: PaymentAndChange {
        payment: DecodedOutput {
            vout: 0,
            value: 1.0,
            address: Some(
                "bcrt1q4kl02lpztynp25lnc50rljr5c9dpf8x8mkxph6",
            ),
            script_pub_key_hex: "0014adbef57c2259261553f3c51e3fc874c15a149cc7",
        },
        change: Some(
            DecodedOutput {
                vout: 1,
                value: 0.1999448,
                address: Some(
                    "bcrt1q6tk5jmd8sktnyrx33ty50mflnld7s9htrtlt8k",
                ),
                script_pub_key_hex: "0014d2ed496da78597320cd18ac947ed3f9fdbe816eb",
            },
        ),
    },
    fee: 5.52e-5,
}`


## Evidence references

https://drive.google.com/drive/folders/1mP1ycuASg9SOfhFiHK00MdBMmprZZjQp?usp=drive_link

## Explanation

Every UTXO a wallet holds sits at its own address, and on its own, a UTXO reveals nothing about who else's coins belong to the same person. Before I spent anything, an outside observer watching the chain would only see three separate 0.4 BTC outputs sitting at three separate addresses — with no way to tell whether they belonged to one person or three different people.

The moment a transaction spends multiple UTXOs as inputs at the same time, that ambiguity disappears. A transaction is only valid if it's signed by the private keys controlling every one of its inputs — so consuming Alice's three 0.4 BTC UTXOs in a single spend is cryptographic proof, visible to anyone on the public chain, that the same person (or wallet) controlled all three simultaneously. My own transaction did exactly this: spend_input_count: 3 and funding_outpoints listing all three previously-separate outpoints together in one transaction is a permanent, public link between them. Anyone doing chain analysis can now say with certainty "these three addresses are commonly owned," something they couldn't have concluded before.

This is the core tension in UTXO-based coin selection: combining inputs is often the only way to reach a required payment amount — Alice genuinely needed all three 0.4 BTC UTXOs to cover a 1 BTC payment, since no single one of her outputs was large enough. But that mechanical necessity comes bundled with an involuntary privacy disclosure: the wallet had no way to spend 1 BTC without also revealing that these specific three UTXOs share an owner. The sender didn't choose to announce "these are all mine" — it was an unavoidable side effect of needing enough value in one place.

This is why privacy-conscious wallets often try to avoid unnecessary UTXO consolidation — for example, preferring to wait for one sufficiently large UTXO, or deliberately structuring earlier transactions to avoid needing to merge many small ones later — even though this can mean higher fees or waiting longer, purely to avoid revealing which coins actually belong together.
