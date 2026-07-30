# Lab 01 — Regtest network inspection

## Commands used

In order to implement and test the regtest network inspection, the following tools and RPC commands were used:

1. **Rust Test Command**:
   ```bash
   cargo test --test lab_01
   ```
2. **Bitcoin Core RPC Commands**:
   - `getblockchaininfo`: Retreives general state information about the blockchain node.
   - `getblockcount`: Returns the number of blocks in the local best block chain.
   - `getbestblockhash`: Returns the hash of the best (tip) block in the local best block chain.

---

## Terminal output

### 1. Verification of the Rust implementation:
Running `cargo test --test lab_01` returns the following output:
```text
running 4 tests
test reads_best_block_hash ... ok
test builds_verified_network_snapshot ... ok
test reads_block_height ... ok
test reads_regtest_chain ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 2. Node RPC responses (Mock representations):
- `getblockchaininfo` response:
  ```json
  {
    "chain": "regtest"
  }
  ```
- `getblockcount` response:
  ```text
  101
  ```
- `getbestblockhash` response:
  ```text
  0000abcd
  ```

---

## Evidence references

- The code implementation for Lab 01 has been written inside [lab01_network.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_1/src/labs/lab01_network.rs).
- Local tests pass successfully, confirming that the client is able to execute, parse, and validate the RPC endpoints correctly against the mock Bitcoin Core clients.

---

## Explanation

Here is the explanation of the roles and differences of the following technologies:

- **Polar**: A developer tool with a graphical user interface (GUI) designed to easily spin up, connect, and configure private networks of Bitcoin Core, LND, and Core Lightning nodes. It makes testing multi-node arrangements and Lightning networks very intuitive.
- **Docker**: The containerization tool that works behind the scenes. Polar uses Docker to deploy and run separate, isolated containers for each Bitcoin or Lightning node. This ensures that every node operates in a clean, isolated environment without interfering with the host machine.
- **Bitcoin Core**: The reference implementation of the Bitcoin protocol. It runs as `bitcoind` (the node daemon) and is controlled/queried via `bitcoin-cli`. It maintains the state of the blockchain, validates transactions and blocks, manages wallet states, and broadcasts peer-to-peer data.
- **Regtest (Regression Test Mode)**: A local, private network mode built into Bitcoin Core. It allows developers to run their own private blockchain, instantly generate blocks on demand without needing high hashing power, and create coins out of thin air. It is perfect for local testing as the coins generated are fake and have no real-world value.
