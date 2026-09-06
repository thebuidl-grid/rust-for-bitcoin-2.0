# Assignment: Building a Bitcoin Wallet in Rust

## Goal

Build a functioning Bitcoin wallet in Rust (regtest) that demonstrates you can use the libraries covered in class effectively.

## Minimum Requirements

Your wallet must be able to:

1. **Generate or import keys** and derive a wallet from a descriptor.
2. **Generate addresses** from both an external (receiving) and internal (change) keychain.
3. **Track UTXOs and calculate balance** for the wallet.
4. **Persist wallet state locally** (e.g. with SQLite) so the wallet can be closed and reopened without losing track of its own state.
5. **Construct, sign, and broadcast a transaction** on testnet.
6. **Connect to a Bitcoin node** (via `bitcoincore-rpc`) to sync wallet state or broadcast transactions, i.e., your wallet should not be purely offline.

## Stretch Goals

Pick any of these if you want to push further:

- Support multiple descriptor types (e.g. compare `wpkh` vs `tr` Taproot)
- Build a simple CLI so a user can check balance, get a new address, and send funds without editing code
- Handle coin selection explicitly rather than relying on defaults
- Add basic error handling/logging that would make this usable by someone other than you
- Explain (in your README) a scenario where you reached for raw `rust-bitcoin` instead of BDK, and demonstrate it with a small code example

## Acceptance Criteria

1. **A PR TO THE RFB GITHUB** (source code, `Cargo.toml`, etc.) as a zip or a link to a repository.
2. **A README** that includes:
   - How to run your wallet (setup instructions, any node/config required)
   - A short explanation (project or descriptor structure, and why you chose it)
   - Which libraries you used where, and why (e.g. "I used `bitcoincore-rpc` for X, and BDK for Y, because...")
   - Any known limitations or things you'd improve with more time

## Constraints

- Testnet or regtest only.
- Do not hardcode private keys or seed phrases in files you submit, use a `.env`, config file, or generate fresh test keys. (This is also good practice for real-world Bitcoin development.)
- You may use any crates that support your chosen libraries (e.g. `dotenv`, `clap`, `tokio`), but the wallet logic itself should go through `rust-bitcoin`, `bitcoincore-rpc`, and/or BDK.

## Reference Material

- rust-bitcoin docs: https://docs.rs/bitcoin/0.32.102/bitcoin/index.html
- Bitcoin Core RPC reference: https://developer.bitcoin.org/reference/rpc/
- BDK Wallet docs: https://docs.rs/bdk_wallet/latest/bdk_wallet/index.html

---

## Grading Rubric (100 points)

| Category | Criteria |
|---|---|
| **Correctness ΓÇö Core Functionality** | Wallet generates keys/addresses correctly from a descriptor; external and internal keychains are properly separated |
| **Correctness ΓÇö UTXO & Balance Tracking** | Wallet accurately tracks UTXOs and reports correct balance after syncing |
| **Correctness ΓÇö Transactions** | Wallet successfully creates, signs, and broadcasts a transaction on testnet/regtest; txid is verifiable |
| **Persistence** | Wallet state survives a restart (SQLite or equivalent) without needing to re-derive/re-sync from scratch |
| **Node Integration** | Wallet correctly connects to and communicates with a Bitcoin node (RPC or BDK-supported connection) |
| **Architecture & Library Use** | Sensible use of rust-bitcoin / bitcoincore-rpc / BDK together, student can justify *why* each was used where|
| **Code Quality** | Reasonably organized, readable, compiles cleanly, handles at least basic errors (doesn't just panic on bad input) |
| **README & Documentation** | Clear setup instructions, explains design decisions, includes proof of a working transaction |
| **Stretch Goals (bonus)** | Any stretch goal attempted and working |

## Timeline

- **Due:** 5th Septermber 2026

## Implementation notes

This implementation uses Bitcoin Core **regtest** so the wallet can be tested locally without real funds.

### Wallet and descriptors

A fresh 32-byte random seed is generated on first use with `getrandom` and persisted in the local SQLite wallet database. The seed is converted to an extended private key and used for two BIP84-style `wpkh` descriptors:

- External/receiving: `wpkh(m/84'/1'/0'/0/*)`
- Internal/change: `wpkh(m/84'/1'/0'/1/*)`

BDK's `KeychainKind::External` and `KeychainKind::Internal` keep receiving and change addresses separate.

### Libraries used

- **BDK Wallet** — descriptors, wallet state, keychains, address derivation, UTXO tracking, balance calculation, transaction construction/signing, and SQLite persistence.
- **bdk_bitcoind_rpc** — synchronizes wallet state with Bitcoin Core blocks and mempool transactions.
- **bitcoincore-rpc** — Bitcoin Core RPC connection, regtest mining, transaction broadcasting, and transaction verification.
- **rust-bitcoin** — Bitcoin network types, addresses, amounts, extended private keys, scripts, and transaction primitives.
- **clap** — command-line interface and configuration.
- **getrandom** — random wallet seed generation.

### Running the wallet

Start the regtest Bitcoin Core node using the project's regtest data directory and ensure RPC is enabled.

The wallet accepts the Bitcoin Core cookie file with `--rpc-cookie`:

```powershell
cargo run -- --rpc-cookie "C:\BitcoinCore\rfb-regtest\regtest\.cookie" demo
Additional commands are available for deriving addresses, synchronizing the wallet, and sending regtest funds:

cargo run -- --rpc-cookie "C:\BitcoinCore\rfb-regtest\regtest\.cookie" address
cargo run -- --rpc-cookie "C:\BitcoinCore\rfb-regtest\regtest\.cookie" address --change
cargo run -- --rpc-cookie "C:\BitcoinCore\rfb-regtest\regtest\.cookie" sync
cargo run -- --rpc-cookie "C:\BitcoinCore\rfb-regtest\regtest\.cookie" send <REGTEST_ADDRESS> <AMOUNT_SAT>

The local wallet.sqlite database is excluded from Git by .gitignore.

Verification evidence
The end-to-end regtest demo was successfully executed and verified:

Connected to Bitcoin Core: regtest
External/receiving address: bcrt1qrjkmyst5x3yhk870lkj9557xtcntvvgmj947zq
Internal/change address:     bcrt1qzap9drtszsmm4d7uu2g34d5n0d8n6k02kaaq0y
Balance before send: 5049.99949718 BTC
Tracked UTXOs: 102
Broadcast txid: f5da63e8bdd56712101b05581cab9525ce2dca088fe39c6d9a50c79bf752f637
Balance after send: { immature: 4950 BTC, trusted_pending: 99.99849577 BTC, untrusted_pending: 0.00100000 BTC, confirmed: 0 BTC }
UTXOs after send: 103
Persistence check: PASS
Transaction verification: PASS (f5da63e8bdd56712101b05581cab9525ce2dca088fe39c6d9a50c79bf752f637)
Demo complete.

The transaction was retrieved from Bitcoin Core after broadcasting and its computed txid matched the broadcast txid. The wallet was also reopened from SQLite and its derivation state was verified.

Validation
The following commands completed successfully:

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo run -- --rpc-cookie "C:\BitcoinCore\rfb-regtest\regtest\.cookie" demo

Known limitations
This is an educational regtest wallet rather than a production wallet.

Only regtest is configured by default.
The generated seed is persisted locally but there is no portable backup/recovery command.
Coin selection relies on BDK's transaction builder.
Only BIP84-style wpkh descriptors are implemented.
Automated unit-test coverage is currently minimal; the end-to-end regtest demo provides the primary functional verification.
Possible future improvements include encrypted backups, explicit coin selection, Taproot descriptors, stronger automated tests, and more detailed logging.
