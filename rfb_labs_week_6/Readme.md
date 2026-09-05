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
| **Correctness — Core Functionality** | Wallet generates keys/addresses correctly from a descriptor; external and internal keychains are properly separated |
| **Correctness — UTXO & Balance Tracking** | Wallet accurately tracks UTXOs and reports correct balance after syncing |
| **Correctness — Transactions** | Wallet successfully creates, signs, and broadcasts a transaction on testnet/regtest; txid is verifiable |
| **Persistence** | Wallet state survives a restart (SQLite or equivalent) without needing to re-derive/re-sync from scratch |
| **Node Integration** | Wallet correctly connects to and communicates with a Bitcoin node (RPC or BDK-supported connection) |
| **Architecture & Library Use** | Sensible use of rust-bitcoin / bitcoincore-rpc / BDK together, student can justify *why* each was used where|
| **Code Quality** | Reasonably organized, readable, compiles cleanly, handles at least basic errors (doesn't just panic on bad input) |
| **README & Documentation** | Clear setup instructions, explains design decisions, includes proof of a working transaction |
| **Stretch Goals (bonus)** | Any stretch goal attempted and working |


---

## Submission

### How to run

**Prerequisites:** Bitcoin Core (`bitcoind`/`bitcoin-cli`) installed, Rust/Cargo installed.

1. Create a regtest data directory and config:
   ```bash
   mkdir -p ~/.bitcoin-regtest
   cat > ~/.bitcoin-regtest/bitcoin.conf << 'EOF'
   regtest=1
   server=1
   fallbackfee=0.0001

   [regtest]
   rpcuser=rfbwallet
   rpcpassword=<choose your own>
   rpcport=18443
   EOF
   ```
2. Start the node and mine some spendable coins into a separate Core wallet (used only as a faucet, not part of this project):
   ```bash
   bitcoind -datadir=$HOME/.bitcoin-regtest -conf=$HOME/.bitcoin-regtest/bitcoin.conf -daemon
   bitcoin-cli -datadir=$HOME/.bitcoin-regtest -rpcuser=rfbwallet -rpcpassword=<...> -regtest createwallet "faucet"
   bitcoin-cli -datadir=$HOME/.bitcoin-regtest -rpcuser=rfbwallet -rpcpassword=<...> -regtest -rpcwallet=faucet \
     generatetoaddress 101 $(bitcoin-cli -datadir=$HOME/.bitcoin-regtest -rpcuser=rfbwallet -rpcpassword=<...> -regtest -rpcwallet=faucet getnewaddress)
   ```
3. Create `.env` in this crate's directory:
   ```
   RPC_URL=http://127.0.0.1:18443
   RPC_USER=rfbwallet
   RPC_PASSWORD=<same as above>
   WALLET_XPRIV=<a BIP32 master extended private key, tprv... for regtest>
   ```
   `WALLET_XPRIV` is generated once (e.g. via `bitcoin::bip32::Xpriv::new_master` from random bytes) and then reused across runs — regenerating it every run would make the wallet forget its own keys. It is never committed; `.env` is gitignored.
4. `cargo run` — connects to the node, loads (or creates) the wallet, reveals an address on each keychain, syncs the chain, and prints the balance.
5. To test a real send, fund the printed external address from the faucet wallet, mine a confirmation, add `SEND_TO_ADDRESS=<a regtest address>` to `.env`, and `cargo run` again — then remove that line so subsequent runs don't keep re-sending.

### Descriptor / architecture

Single-sig native SegWit (`wpkh`), BIP84 derivation path (`m/84'/1'/0'/{0,1}/*`), coin type `1` for testnet/regtest. Two descriptor strings share the same master key and account level, differing only in the change-level digit (`0` = external/receiving, `1` = internal/change) — this is the standard convention every BIP44-family wallet uses, and BDK's `Wallet::create(external, internal)` takes exactly this pair, keeping the two keychains cleanly separated at the type level (`KeychainKind::External` / `::Internal`).

Three components hand off to each other:
- `bitcoincore_rpc::Client` — the only thing that talks to the node directly: fetching chain data and broadcasting the final signed transaction.
- `bdk_bitcoind_rpc::Emitter` — polls that client and turns raw node data into block/mempool events shaped for BDK to consume (`wallet.apply_block(...)`, `wallet.apply_unconfirmed_txs(...)`).
- `bdk_wallet::Wallet` — never talks to the network itself; it only consumes update events and produces addresses/balances/PSBTs.

### Which library did what, and why

- **`bitcoincore-rpc`** — raw node connectivity: `get_blockchain_info` (connectivity check), and `send_raw_transaction` (broadcast). Chosen because it's the direct, unopinionated binding to Bitcoin Core's RPC — no wallet-RPC dependency, works against a wallet-disabled node.
- **`bdk_bitcoind_rpc`** (`Emitter`) — bridges `bitcoincore-rpc` to BDK's chain-tracking data structures via polling (`next_block`/`mempool`), so the wallet doesn't need its own node-talking code.
- **`bdk_wallet`** — descriptor parsing and key derivation, address generation per keychain, UTXO/balance tracking, PSBT construction and signing, and SQLite-backed persistence (`rusqlite` feature). This is the core "wallet logic" library; almost everything routes through it.
- **`bitcoin` (rust-bitcoin)** — used directly for `Xpriv` (master key parsing/generation), `Network`, `Address` (parsing the send destination), and `Amount`. BDK re-exports much of this, but these specific types were used at the call sites that needed them directly (e.g. parsing `WALLET_XPRIV` before it's interpolated into a descriptor string).
- **`rusqlite`** (via `bdk_wallet`'s own re-export, not a separate dependency) — the actual SQLite engine backing persistence; using the re-export rather than an independently versioned `rusqlite` avoids two incompatible copies of the crate existing at once.
- **`anyhow`** — `main` calls into three crates with three different error types (`bitcoincore_rpc::Error`, `bdk_wallet`'s various error types, `rusqlite::Error`); `anyhow::Result` lets `?` work uniformly across all of them without hand-writing a unifying error enum for a single-binary lab project.
- **`dotenvy`** — loads `.env` at startup so RPC credentials and the wallet's private key never appear in source, satisfying the assignment's no-hardcoded-secrets constraint.

### Known limitations / would improve with more time

- Only single-sig `wpkh` (BIP84) is supported — no Taproot or multisig (stretch goal not attempted).
- Coin selection relies entirely on BDK's default algorithm; no explicit UTXO selection logic.
- No CLI — behavior is controlled by which env vars are set (`SEND_TO_ADDRESS` gates the send path) rather than real subcommands/arguments.
- Error handling is `?`/`anyhow` throughout rather than domain-specific error types with actionable messages — fine for a lab, not production-shaped.
- `Emitter` always starts a full scan from block height 0; on regtest that's instant, but a real deployment would want to start from the wallet's actual birthday height.
- No fee bumping / RBF support.
- One non-obvious gotcha worth documenting: `Wallet::load().load_wallet(&mut conn)` alone reconstructs a **watch-only** wallet — it does not restore signing capability even when the original descriptor had a private key embedded. Restoring the signer after a reload requires explicitly chaining `.descriptor(keychain, Some(desc))` and `.extract_keys()` before `.load_wallet(...)`. This cost real debugging time (a transaction built and "signed" with `finalized: false`, silently producing an empty witness, rejected by the node with a script-verification error) before being traced to this.

### Example output

```
$ cargo run
chain: regtest
block: 103
External Address: bcrt1qdtmpyl89ag0u5sa3sel4fv5eepl3vhxcct0p5n
Internal Address: bcrt1qg287395cdc00vsfs4xl68fk8yn48s094r4hxtr
balance: 0.99899718 BTC
```

### Proof of a working transaction

Txid `1c493ea0dc29dc0f2360e33998e6b0ec4a383fc59c5c1a238a06c16348da0e8f`, confirmed at regtest block height 103:

```
$ bitcoin-cli ... gettransaction 1c493ea0dc29dc0f2360e33998e6b0ec4a383fc59c5c1a238a06c16348da0e8f
{
  "amount": 0.00050000,
  "confirmations": 1,
  "blockheight": 103,
  ...
}
```

## Timeline

- **Due:** 5th Septermber 2026