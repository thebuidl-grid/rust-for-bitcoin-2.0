# Building a Bitcoin Wallet in Rust

A simple Bitcoin wallet built in Rust using the Bitcoin Rust ecosystem. The wallet runs on **regtest** and demonstrates key generation, descriptor-based wallet creation, address derivation, UTXO tracking, persistence, blockchain synchronization, transaction signing, and broadcasting.

## Project Structure

```text
rfb_labs_week_6/
├── src/
│   └── main.rs
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── Readme.md
└── EVIDENCE.md
````

The implementation is intentionally small and keeps the main wallet flow in `main.rs` so the individual Bitcoin operations are easy to follow.

## Architecture

```text
                    Polar Bitcoin Core
                           |
                           | RPC
                           v
                    bitcoincore-rpc
                           |
                           v
                    bdk_bitcoind_rpc
                           |
                           | sync blocks/mempool
                           v
                       BDK Wallet
                    +------+------+
                    |             |
               External       Internal
                 /0/*            /1/*
                    |             |
                    +------+------+
                           |
                        SQLite
```

The wallet uses BIP84-style SegWit descriptors:

```text
External: wpkh(<xprv>/84'/1'/0'/0/*)
Internal: wpkh(<xprv>/84'/1'/0'/1/*)
```

The external keychain is used for receiving addresses and the internal keychain is used for change addresses.

## Libraries

### BDK Wallet

`bdk_wallet` provides the higher-level wallet functionality:

* Descriptor-based wallet creation
* External and internal keychains
* Address derivation
* UTXO tracking
* Balance calculation
* Transaction construction
* Transaction signing
* SQLite persistence

BDK was used because it provides the wallet-level functionality required by the assignment without having to implement wallet state management from scratch.

### Bitcoin Core RPC

`bitcoincore-rpc` connects the application to the Bitcoin Core node running in Polar.

It is used to:

* Connect to Bitcoin Core
* Query blockchain information
* Broadcast signed transactions

### BDK Bitcoind RPC

`bdk_bitcoind_rpc` provides the synchronization bridge between Bitcoin Core and the BDK wallet.

It is used to stream blocks and mempool transactions into the wallet so that BDK can update its UTXO and balance state.

### rust-bitcoin

The Bitcoin types exposed through BDK are used for Bitcoin-specific primitives such as:

* `Network`
* `Address`
* `Amount`
* BIP32 extended private keys

## Setup

### Requirements

* Rust and Cargo
* Polar
* A running Bitcoin Core regtest node inside Polar

This project was tested with a Polar Bitcoin Core node using:

```text
Network: regtest
RPC URL: http://127.0.0.1:18443
```

### Environment Variables

Create a `.env` file inside `rfb_labs_week_6`:

```env
RPC_URL=http://127.0.0.1:18443
RPC_USER=<your_rpc_user>
RPC_PASSWORD=<your_rpc_password>
WALLET_XPRV=<your_regtest_xprv>
```

The `.env` file is ignored by Git and should not be committed.

To generate a fresh disposable regtest extended private key:

```bash
cargo run -- generate-key
```

Copy the generated key into `.env` as `WALLET_XPRV`.

## Running the Wallet

### Show wallet status

```bash
cargo run -- status
```

This connects to Bitcoin Core, synchronizes the wallet, and displays:

* Current balance
* A receiving address
* A change address
* Wallet UTXOs

### Generate a new key

```bash
cargo run -- generate-key
```

This generates a fresh random regtest extended private key.

### Send funds

```bash
cargo run -- send <regtest-address> <amount>
```

For example:

```bash
cargo run -- send bcrt1... 1.0
```

The wallet:

1. Validates the recipient address and amount.
2. Builds a transaction using BDK.
3. Selects available UTXOs.
4. Creates a PSBT.
5. Signs the PSBT.
6. Extracts the final transaction.
7. Broadcasts it through Bitcoin Core.
8. Prints the resulting transaction ID.

## Persistence

Wallet state is stored locally in:

```text
wallet.sqlite
```

The database is ignored by Git.

On startup, the application attempts to load the existing wallet state from SQLite. If no wallet exists, it creates one from the descriptors.

This allows the wallet to be closed and reopened while retaining its local wallet state.

## Transaction Flow

The transaction flow is:

```text
BDK TxBuilder
     |
     v
   PSBT
     |
     v
BDK wallet.sign()
     |
     v
Final transaction
     |
     v
Bitcoin Core RPC
     |
     v
Regtest mempool
     |
     v
Confirmed in a mined block
```

A working transaction was constructed, signed, broadcast, and confirmed on the local regtest network.

See [EVIDENCE.md](EVIDENCE.md) for the transaction evidence.

## Known Limitations

This is a learning-focused wallet rather than a production wallet.

Current limitations include:

* Only regtest is supported.
* The wallet currently uses a single BIP84 `wpkh` descriptor structure.
* Coin selection is delegated to BDK's default coin selection algorithm.
* The CLI is intentionally minimal.
* Wallet configuration is provided through environment variables rather than a dedicated configuration system.
* There is no encrypted wallet database or hardware-wallet support.
* There are no recovery or backup workflows beyond retaining the wallet's extended private key.


## Assignment Requirements

The implementation demonstrates:

* Key generation and descriptor-based wallet creation
* External and internal keychains
* UTXO tracking and balance calculation
* SQLite persistence
* Bitcoin Core RPC integration
* Blockchain synchronization
* Transaction construction
* Transaction signing
* Transaction broadcasting
* Verifiable regtest transaction confirmation

````
