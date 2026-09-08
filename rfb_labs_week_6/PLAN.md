# Build Plan — Bitcoin Wallet in Rust (regtest)

Working plan for the Week 6 assignment. Tick boxes as you go.

- **Started:** 2026-09-08
- **Assignment:** [`Readme.md`](./Readme.md)
- **Current phase:** Phase 1 ✅ complete — next: Phase 2 (node sync)

---

## 1. Locked decisions

| Concern | Choice | Why |
|---|---|---|
| Network | **regtest** | Instant blocks, no faucet. Assignment permits testnet *or* regtest. |
| Node | Polar network `week6_wallet` → container `polar-n2-backend1`, Core **30.0** | Verified working. Ships `-fallbackfee` already set. |
| RPC | `http://127.0.0.1:18443`, user `polaruser`, pass `polarpass` | Polar uses `-rpcauth`; **no cookie file exists**, so auth must be user/pass. |
| Wallet engine | `bdk_wallet 3.1` | Descriptor expansion, SPK index, UTXO set, coin selection, PSBT signing, changeset persistence. |
| Chain source | `bdk_bitcoind_rpc 0.22` (`Emitter`) | Handles reorgs **and** mempool. Re-exports a version-matched `bitcoincore-rpc 0.19`. |
| Primitives | `bdk_wallet::bitcoin` (0.32.102) | **Never add `bitcoin` to Cargo.toml yourself** — causes a version split. |
| Persistence | `bdk_wallet::rusqlite` (0.31) | **Never add `rusqlite` yourself** — resolves to 0.40 and conflicts. |
| Descriptor | BIP84 `wpkh` default; BIP86 `tr` behind a config flag | `tr` is a stretch goal for ~20 extra lines. |
| Core's own wallet | **Unused** | Core is a chain source + broadcaster only. This is a key README argument. |

### The version trap (already avoided)

`bdk_wallet` and `bitcoincore-rpc` both depend on `bitcoin`. If they resolve to different
majors, every shared type (`Transaction`, `Txid`, `BlockHash`) becomes a *distinct* type and you
get "expected `bitcoin::Transaction`, found `bitcoin::Transaction`". Verified resolution:

```
bitcoin v0.32.102
├── bdk_bitcoind_rpc v0.22.0
├── bdk_chain v0.23.3        └── bdk_wallet v3.1.0
├── bdk_core v0.6.3
├── bitcoincore-rpc-json v0.19.0 └── bitcoincore-rpc v0.19.0
└── miniscript v12.3.7
```

One `bitcoin`, and it matches the version the assignment's reference docs link to.

---

## 2. Verified environment

| Thing | Status |
|---|---|
| `rustc` / `cargo` | 1.96.0 |
| Polar | v4.0.0 AppImage, `~/Downloads/polar-linux-x86_64-v4.0.0.AppImage` |
| Network | id=2 `week6_wallet`, **Started** |
| Node | `polar-n2-backend1`, Core 30.0.0, regtest |
| RPC port | `18443` (host) |
| `txindex` | synced |
| `basic block filter index` | synced |
| Image | `polarlightning/bitcoind:30.0` cached locally (267MB) |

### Compatibility question — RESOLVED

`bitcoincore-rpc 0.19` predates Core 30, and it deserializes RPC replies into fixed structs.
The known breaking change is Core 28 turning `warnings` from a string into an array.

- Node returns `"warnings": []` (array) — confirmed live.
- `bitcoincore-rpc-json 0.19` types it as `StringOrStringArray` — handles both.
- `Emitter` only calls: `get_block_hash`, `get_block_header_info`, `get_block_info`,
  `get_block`, `get_raw_mempool`. It never calls `getblockchaininfo`.
- Result structs have no `deny_unknown_fields`, so *added* fields are ignored.

**Core 30.0 works. No fallback needed.** (If it ever breaks: recreate the Polar node at 27.0.)

---

## 3. Module layout

```
rfb_labs_week_6/
├── Cargo.toml
├── .env                 # gitignored — holds the mnemonic
├── .env.example         # committed — keys, no values
├── .gitignore
├── PLAN.md              # this file
├── README.md            # deliverable
└── src/
    ├── main.rs          # dispatch + top-level error printing
    ├── cli.rs           # clap derive structs
    ├── config.rs        # .env → Config; rejects mainnet
    ├── error.rs         # thiserror enum + Result<T>
    ├── keys.rs          # mnemonic ↔ xprv ↔ descriptors
    ├── wallet.rs        # load-or-create PersistedWallet, addresses, balance
    ├── node.rs          # RPC client + Emitter sync loop
    └── tx.rs            # build / sign / broadcast
```

---

## 4. Phases

### Phase 0 — Node  ✅ DONE

- [x] Polar installed, network `week6_wallet` created and started
- [x] `bitcoind` 30.0 reachable on `127.0.0.1:18443`
- [x] Credentials confirmed (`polaruser` / `polarpass`)
- [x] RPC smoke test passed; `bitcoincore-rpc 0.19` compatibility confirmed

### Phase 1 — Keys, descriptors, addresses, persistence

Files: `Cargo.toml`, `error.rs`, `config.rs`, `keys.rs`, `wallet.rs`, `cli.rs`, `main.rs`

- [x] `Cargo.toml` with the five dependencies (see §5)
- [x] `.gitignore` — `.env`, `*.sqlite`, `/target`
- [x] `error.rs` — thiserror enum + `pub type Result<T>`
- [x] `config.rs` — load `.env`, parse network, **reject mainnet**
- [x] `keys.rs` — generate / parse mnemonic → xprv → descriptors
- [x] `wallet.rs` — load-or-create `PersistedWallet<Connection>`
- [x] `cli.rs` + `main.rs` — `init`, `info`, `address new|list`, `balance`, `utxos`
- [x] **Checkpoint** — verified live: index continued at 2 after restart

Key API path:

```rust
Mnemonic::generate((WordCount::Words12, Language::English))
  → (mnemonic, passphrase).into_extended_key()?          // ExtendedKey<Segwitv0>
  → .into_xprv(NetworkKind::from(network))               // Option<Xpriv>
  → Bip84(xprv, KeychainKind::External).build(network_kind)?   // (desc, keymap, _)
  → desc.to_string_with_secret(&keymap)                  // private descriptor string
  → desc.to_string()                                     // public / watch-only
```

Load-or-create:

```rust
let mut wallet = match Wallet::load()
    .descriptor(KeychainKind::External, Some(external.clone()))
    .descriptor(KeychainKind::Internal, Some(internal.clone()))
    .extract_keys()
    .check_network(network)
    .load_wallet(&mut conn)?
{
    Some(w) => w,
    None => Wallet::create(external, internal)
        .network(network)
        .create_wallet(&mut conn)?,
};
```

Three things that bite later if skipped:

1. **`.extract_keys()`** — without it, a reloaded wallet holds *public* descriptors only. It
   can watch but cannot sign. Surfaces in Phase 3 as `sign()` returning `false`.
2. **`.check_network()`** — stops you opening a testnet DB with regtest config.
3. **`wallet.persist(&mut conn)` after every mutation** — `reveal_next_address` bumps the
   index *in memory* only.

> **Checkpoint:** `address new` twice → indices 0, then 1. **Kill the process. Run again →
> index 2, not 0.** This is literally the Persistence rubric row. Failing it means a missing
> `persist()`.

Rubric: *Core Functionality*, *Persistence*

### Phase 2 — Sync

Files: `node.rs`, extend `wallet.rs`

- [ ] `Client::new(url, Auth::UserPass(user, pass))`
- [ ] `Emitter::new(client, wallet.latest_checkpoint(), start_height, mempool_txs)`
- [ ] Loop `emitter.next_block()` → `wallet.apply_block_connected_to(&block, height, connected_to)`
- [ ] Then `emitter.mempool()` → `wallet.apply_unconfirmed_txs(..)`
- [ ] `persist()` after the loop
- [ ] `sync`, `balance`, `utxos` commands
- [ ] `fund --blocks 101` helper
- [ ] **Checkpoint** (see below)

Funding on regtest — no faucet, no node wallet required:

```bash
bitcoin-cli -regtest -rpcconnect=127.0.0.1 -rpcport=18443 \
  -rpcuser=polaruser -rpcpassword=polarpass \
  generatetoaddress 101 <your external address>
```

> **Expect the balance to land in `immature` first.** Coinbase outputs need 100 confirmations.
> Mining 101 blocks matures exactly the first one → 50 BTC spendable (regtest subsidy is
> 50 BTC for the first 150 blocks). This is not a bug — explain it in the README.

> **Checkpoint:** `sync` then `balance` shows ~50 BTC confirmed; `utxos` lists the coinbase
> outputs with keychain + derivation index.

Rubric: *UTXO & Balance Tracking*, *Node Integration*

### Phase 3 — Spend

Files: `tx.rs`

- [ ] `wallet.build_tx()` → `.add_recipient(spk, amount)` → `.fee_rate(fee_rate)` → `.finish()` → PSBT
- [ ] `wallet.sign(&mut psbt, SignOptions::default())` — **assert it returns `true`**
- [ ] `psbt.extract_tx()?` → `client.send_raw_transaction(&tx)`
- [ ] `persist()` after broadcast
- [ ] `send --to --amount [--fee-rate]` command
- [ ] **Checkpoint** (see below)

> **Checkpoint:** txid verifiable via `getrawtransaction <txid> true`. Mine 1 block, re-sync,
> confirm the change output landed on the **internal** keychain. Save the terminal output —
> it's a required README artifact.

Rubric: *Transactions*

### Phase 4 — README + stretch

Required by the acceptance criteria:

- [ ] How to run it (Polar setup, `.env`, commands)
- [ ] Project + descriptor structure, and why
- [ ] Which library used where, and why
- [ ] Known limitations / what you'd improve
- [ ] **Proof of a working transaction** (txid + terminal output)
- [ ] PR to the RFB GitHub

Stretch goals, ranked by value per unit effort:

- [ ] **CLI** — free, it's the spine of the project
- [ ] **`wpkh` vs `tr`** — one enum + one match arm; show both descriptors and address forms
- [ ] **Raw `rust-bitcoin` vs BDK** — the `Xpriv` → `Bip84` step, or reuse Week 3's `decodetrx`
- [ ] **Explicit coin selection** — swap default for `LargestFirstCoinSelection`
- [ ] **Error handling / logging** — `error.rs` already covers most of this

---

## 5. Cargo.toml dependencies

```toml
[dependencies]
bdk_wallet       = { version = "3.1", features = ["rusqlite", "keys-bip39"] }
bdk_bitcoind_rpc = "0.22"
clap             = { version = "4.5", features = ["derive"] }
dotenvy          = "0.15"
thiserror        = "2"
```

Nothing else. In particular **not** `bitcoin` and **not** `rusqlite` — reach them through
`bdk_wallet::bitcoin` and `bdk_wallet::rusqlite`.

### What the SQLite database actually contains (verified)

The `bdk_wallet` table stores the **public** descriptor only:

```
descriptor = wpkh([dbd66f9f/84'/1'/0']tpubDC6pm1sxUWCWp.../0/*)
```

No `tprv` anywhere in the file. Consequences:

* Losing the database costs a resync, never the coins. The seed is the only
  irreplaceable artifact.
* It is still gitignored, and chmod'd to 0600 on open — it holds the account
  xpub plus every address and transaction, a full privacy picture.
* **`.extract_keys()` matters for a different reason than "the DB stores
  secrets".** Signing keys come from the private descriptors passed to
  `.descriptor(...)` at load time. Without `extract_keys` BDK verifies against
  them and discards the secrets. Measured:

  | load path | signing keys in key map |
  |---|---|
  | freshly created | 1 |
  | reloaded **without** `extract_keys` | **0** |
  | reloaded **with** `extract_keys` | 1 |

### Verified type paths (compile-tested)

```rust
use bdk_wallet::bip39::Error as Bip39Error;          // needs feature "keys-bip39"
use bdk_wallet::keys::KeyError;
use bdk_wallet::descriptor::DescriptorError;         // = descriptor::error::Error
use bdk_wallet::rusqlite;                            // rusqlite 0.31, re-exported
use bdk_wallet::{CreateWithPersistError, LoadWithPersistError};
use bdk_bitcoind_rpc::bitcoincore_rpc;               // version-matched client
use bdk_wallet::bitcoin::network::ParseNetworkError;
```

`#[from]` works on `CreateWithPersistError<rusqlite::Error>` and
`LoadWithPersistError<rusqlite::Error>` — they are concrete once `E` is fixed, and BDK
implements `Error` for them where `E: Debug + Display`. No manual `impl From` needed.


---

## 6. CLI surface

```
init                              generate mnemonic → .env
info                              descriptors, fingerprint, network, db path
address new [--change]            reveal next on external / internal
address list                      all revealed, both keychains
sync                              pull blocks + mempool from the node
balance                           confirmed / immature / pending
utxos                             outpoint, value, keychain, derivation index
send --to <addr> --amount <sat>   [--fee-rate <sat/vb>]
fund --blocks 101                 regtest: generatetoaddress to own address
```

---

## 7. .env keys

```
BITCOIN_NETWORK=regtest
WALLET_MNEMONIC=            # generated by `init` — never commit
WALLET_PASSPHRASE=          # optional BIP39 passphrase
WALLET_DB=./wallet.sqlite
DESCRIPTOR_KIND=wpkh        # wpkh | tr
RPC_URL=http://127.0.0.1:18443
RPC_USER=polaruser
RPC_PASSWORD=polarpass
```

---

## 8. Tests

| Test | Asserts | Rubric row |
|---|---|---|
| `descriptors_are_deterministic` | fixed mnemonic → known descriptor string + address | Core Functionality |
| `keychains_are_separate` | external `/0/*`, internal `/1/*`, no address overlap | Core Functionality |
| `persistence_survives_restart` | reveal 3 → drop → reopen → index is 3 | **Persistence** |
| `mainnet_is_rejected` | `BITCOIN_NETWORK=bitcoin` → clean error, no panic | Code Quality |

A fixed test mnemonic in `tests/` is fine — the constraint is about *your* keys, not published
test vectors. Note that in the README.

---

## 9. Risk register

| Risk | Status |
|---|---|
| `bitcoin` version split across BDK / bitcoincore-rpc | ✅ resolved — single 0.32.102 |
| Core 30 vs `bitcoincore-rpc 0.19` | ✅ resolved — `warnings` array handled |
| No fee estimation on regtest | ✅ Polar sets `-fallbackfee=0.0002` |
| Reloaded wallet cannot sign | ⚠️ needs `.extract_keys()` — bites in Phase 3 |
| Address index resets on restart | ⚠️ needs `persist()` after every reveal |
| IBus breaks GUI text input | ⚠️ worked around at launch; `ibus restart` is the real fix |

---

## 10. Sequencing

Assignment README says due **5 Sept 2026**; today is **8 Sept 2026** — confirm the deadline
with the instructor.

Minimum viable, in order: **Phase 1 → Phase 2 → Phase 3 → README.**
Stretch goals are bonus only. A working broadcast txid is worth far more than a Taproot
comparison. Don't start §4 stretch until `send` works end to end.

---

## Appendix — useful commands

```bash
# Talk to the Polar node
alias bcli='bitcoin-cli -regtest -rpcconnect=127.0.0.1 -rpcport=18443 \
  -rpcuser=polaruser -rpcpassword=polarpass'
bcli getblockchaininfo
bcli generatetoaddress 101 <address>
bcli getrawtransaction <txid> true

# Polar containers
docker ps --filter name=polar

# Relaunch Polar with the IBus workaround (text input dies without this)
env -u QT_IM_MODULE XMODIFIERS=@im=none GTK_IM_MODULE=gtk-im-context-simple \
  DISPLAY=:0 ~/Downloads/polar-linux-x86_64-v4.0.0.AppImage &
```
