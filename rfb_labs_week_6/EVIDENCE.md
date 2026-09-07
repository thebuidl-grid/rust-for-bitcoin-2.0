# Week 6 Wallet — Evidence of Functionality

All wallet activity was performed on a local Bitcoin Core regtest node running in Polar.

## 1. Bitcoin Core Connection

The wallet successfully connected to Bitcoin Core through `bitcoincore-rpc`.

    Connected to Bitcoin Core: regtest at height 214

## 2. Wallet Synchronization

The wallet successfully loaded its existing SQLite state and synchronized with Bitcoin Core.

    Loaded existing wallet.
    Syncing wallet...
    Wallet synced.

## 3. Balance and UTXO Tracking

After synchronization, the wallet reported:

    Balance: 3448.99999859 BTC

The wallet also listed its tracked UTXOs.

The change output from the completed transaction was present in the wallet:

    4331c85c88e5a5403ac2b6be4e19fa170c23e8dd62106eb760d92d8b222ff5b9:1 — 48.99999859 BTC

This demonstrates that the wallet tracked the transaction output after confirmation and synchronization.

## 4. Transaction Construction and Signing

The wallet successfully constructed and signed a PSBT:

    === Transaction ===
    PSBT created successfully.
    Inputs: 1
    Outputs: 2
    Transaction signed: true

The transaction contained:

- 1 BTC recipient output
- 48.99999859 BTC change output

## 5. Broadcast

The signed transaction was broadcast through Bitcoin Core.

Transaction ID:

    4331c85c88e5a5403ac2b6be4e19fa170c23e8dd62106eb760d92d8b222ff5b9

## 6. Confirmation

The transaction was subsequently mined into the local regtest chain.

Block hash:

    38fb9770e981f6aca40d67c1dd554566eb5a3877f8b0f2273988c4016a094507

Confirmation count:

    1

## 7. End-to-End Flow

The completed flow was:

    Construct transaction
            |
            v
          PSBT
            |
            v
        Sign PSBT
            |
            v
      Extract transaction
            |
            v
       Broadcast via RPC
            |
            v
      Regtest mempool
            |
            v
         Mine block
            |
            v
       Transaction confirmed
            |
            v
       Sync wallet again
            |
            v
     Track change UTXO

## 8. Code Validation

The project was formatted with:

    cargo fmt

The project successfully compiled with:

    cargo check

The wallet status command was also successfully tested:

    cargo run -- status

The status command connects to Bitcoin Core, synchronizes the wallet, and displays the current balance, addresses, and tracked UTXOs.

Transactions are only created and broadcast explicitly using:

    cargo run -- send <regtest-address> <amount>

## 9. Summary

The implementation demonstrates a complete working Bitcoin wallet flow on regtest:

- Bitcoin Core RPC connection
- Descriptor-based wallet creation
- External receiving addresses
- Internal change addresses
- UTXO tracking
- Balance calculation
- SQLite persistence
- Blockchain synchronization
- PSBT construction
- Transaction signing
- Transaction broadcasting
- Transaction confirmation
- Post-confirmation wallet synchronization