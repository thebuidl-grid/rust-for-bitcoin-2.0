# Transaction Serializer

Serializes a Bitcoin transaction from a JSON file to raw hex.

## Usage

```bash
cargo run -- --file <path_to_json>

test@pop-os:~/Desktop/rust/rust-for-bitcoin-2.0/rfb_labs_week_3/serializeTrx$ cargo run -- --file tx.json
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/serialize_trx --file tx.json`
Serialized Hex transaction:
020000000001018fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc8210100000000ffffffff02fe10010000000000160014a632c1fff47af29f8c81dc4c6e91eb49a116c12b02730000000000001600149831122b93d21715c70db626ccc844d3c21f968702483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb235800000000

Transaction size: 223 bytes
```

## Example

```bash
cargo run -- --file examples/p2wpkh.json
```
