# Lab 06 — Transaction decoding

## Commands used
cargo test --test lab_06
docker exec polar-n3-backend1 bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass generatetoaddress 1 bcrt1qq6jewkpw6yv97xpxkt8yf2j33p68fhe7kn4sfc
cargo run --example lab06_check


## Terminal output

`decoded transaction: DecodedTransaction {
    txid: "a9d0febd729cf46b33a44e7a2007266ac1332b554cfd6f98aae864036701aaa9",
    inputs: [
        DecodedInput {
            previous_output: OutPoint {
                txid: "dda96d926694608d4699a9858ec6ac324f9633a7a3a64562aa76389285ca1be3",
                vout: 0,
            },
            previous_value: 50.0,
        },
    ],
    outputs: [
        DecodedOutput {
            vout: 0,
            value: 48.9999718,
            address: Some(
                "bcrt1qxrlfyqfkjvkgzpn9ucdprxmr27p2uyw6fyqs2h",
            ),
            script_pub_key_hex: "001430fe920136932c810665e61a119b635782ae11da",
        },
        DecodedOutput {
            vout: 1,
            value: 1.0,
            address: Some(
                "bcrt1q0uwx0lqm5p8geyd0njz0hjmcl5fnrdl6ka56at",
            ),
            script_pub_key_hex: "00147f1c67fc1ba04e8c91af9c84fbcb78fd1331b7fa",
        },
    ],
    vsize: 141,
}
consumed outpoints: [
    OutPoint {
        txid: "dda96d926694608d4699a9858ec6ac324f9633a7a3a64562aa76389285ca1be3",
        vout: 0,
    },
]
payment and change: PaymentAndChange {
    payment: DecodedOutput {
        vout: 1,
        value: 1.0,
        address: Some(
            "bcrt1q0uwx0lqm5p8geyd0njz0hjmcl5fnrdl6ka56at",
        ),
        script_pub_key_hex: "00147f1c67fc1ba04e8c91af9c84fbcb78fd1331b7fa",
    },
    change: Some(
        DecodedOutput {
            vout: 0,
            value: 48.9999718,
            address: Some(
                "bcrt1qxrlfyqfkjvkgzpn9ucdprxmr27p2uyw6fyqs2h",
            ),
            script_pub_key_hex: "001430fe920136932c810665e61a119b635782ae11da",
        },
    ),
}
fee: 0.00002820000000269829
value conservation: 50 = 49.9999718 + 0.00002820000000269829 (sum(inputs) = sum(outputs) + fee)`


## Evidence references

https://drive.google.com/drive/folders/1mP1ycuASg9SOfhFiHK00MdBMmprZZjQp?usp=drive_link

## Explanation

`Bitcoin's transaction format has no field anywhere for "fee." A transaction only declares two things: which previous outputs it consumes (inputs) and where the value goes (outputs). The fee isn't written down at all — it's an emergent quantity, computed as sum(inputs) - sum(outputs), and it only exists as an inference the node makes after the fact.

My own decoded transaction shows this precisely: the single input consumed 50.0 BTC (the matured coinbase UTXO from Lab 04). The outputs explicitly assign 1.0 BTC to the receiver and 48.9999718 BTC back to miner as change — together, 49.9999718. Nobody wrote 0.0000282 anywhere in the transaction; that number only appears because I computed 50.0 - 49.9999718 myself in calculate_fee. Bitcoin Core independently confirmed the same value when it exposed "fee" in the verbose-2 decode. The fee is real money that moved, but it was never assigned a destination inside the transaction structure — it's simply whatever value the sender chose not to claim in an output.

This design is what makes fees work as a market instead of a fixed cost. If the fee had to be a dedicated output, the sender would need to name a specific recipient address for it at the moment they signed the transaction — but they don't know in advance which miner will eventually include their transaction in a block; that's decided later, by open competition among miners choosing which transactions to include. By instead defining the fee as "whatever's left unassigned," any miner who successfully mines the block containing this transaction automatically collects that leftover value as part of their own coinbase reward — no need for the sender to pre-name a recipient, and no need for miners to coordinate in advance about who gets paid.`
