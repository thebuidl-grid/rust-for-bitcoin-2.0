# Lab 01 — Regtest network inspection

## Commands used

### Creating a Bitcoin Core node and starting the network

Within Polar:
- Click "Create Network"
- Call it "Week 1 Bitcoin Fundamentals"
- Set the number of Bitcoin Core nodes to 1. All the other options should be set to 0.
- Slick "Start"
- Wait until "Started" status

Whithin the project:

cargo build
cargo run --bin lab_01_usage


## Terminal output

get_chain: regtest
get_block_height: 0
get_best_block_hash: 0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206
inspect_network: 
    NetworkSnapshot { 
        chain: "regtest", 
        block_height: 0, 
        best_block_hash: "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206" 
    }

## Evidence references

get_chain returns the blockchain type: in this case regtest.
get_block_height returns current block height.
get_best_block_hash returns best block hash
inspect_network return all previous data consolidated

## Explanation

Polar: A development toolset used to run crypto nodes.
Docker: A a tool to package and run software in isolated containers, making  environments easily reproducible.
Bitcoin Core: The reference implementation of the Bitcoin protocol. It provides the full node software with RPCs, block/transaction validation, and peer-to-peer networking.
regtest: A  testing mode in Bitcoin Core where is possible to create blocks, mine, set network parameters, etc and can test scenarios quickly without real-world network conditions.