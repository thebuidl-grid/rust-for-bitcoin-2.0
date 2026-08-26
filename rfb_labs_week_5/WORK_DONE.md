# Work Done — Week 5

## Project Summary

This document tracks the work completed for the Rust for Bitcoin 2.0 Week 5 assignment:
10 practical labs on Bitcoin address formats and HD wallets (BIP32/BIP39/BIP44/BIP49/BIP84),
implemented with `rust-bitcoin` and `bip39` against the published public BIP39 test
mnemonic only. No Bitcoin Core, Polar, Docker, or live funds were used.

## Repository

- **GitHub URL**: https://github.com/nzubepolycap-hub/rust-for-bitcoin-2.0
- **Branch**: `rust-for-bitcoin-5.0`
- **Local path**: `/home/blackghost/Documents/rust-for-bitcoin-2.0/rust-for-bitcoin-2.0/rfb_labs_week_5`
- **Upstream assignment**: https://github.com/thebuidl-grid/rust-for-bitcoin-2.0/tree/main/rfb_labs_week_5

## What Was Accomplished

### 1. Fetched the assignment scaffold

The `rfb_labs_week_5/` folder did not exist in this fork yet (the fork had weeks 1–3
only). Pulled the full, unmodified starter scaffold (42 files: `Cargo.toml`, `LABS.md`,
`README.md`, `src/`, `tests/`, `submissions/`, `grader/`, CI workflow) directly from the
upstream `thebuidl-grid/rust-for-bitcoin-2.0` repository via the GitHub API, so the
starting point matches the assignment exactly.

### 2. Implemented all 10 labs (`src/labs/`)

Every function that was `todo!()` in the starter was implemented against the public
test suite in `tests/lab_01.rs` .. `tests/lab_10.rs`, without changing any public
signature:

| Lab | File | What it does |
|---:|---|---|
| 01 | `lab01_addresses.rs` | Prefix heuristics, network-checked address parsing (`require_network`), scriptPubKey extraction |
| 02 | `lab02_p2pkh.rs` | P2PKH address derivation, `OP_DUP OP_HASH160 ... OP_EQUALVERIFY OP_CHECKSIG` construction, ScriptSig template |
| 03 | `lab03_p2sh.rs` | 2-of-3 `OP_CHECKMULTISIG` redeemScript, P2SH address, outer `OP_HASH160 ... OP_EQUAL` lock |
| 04 | `lab04_p2wpkh.rs` | Native SegWit v0 witness program, `0 <hash>` scriptPubKey, empty-ScriptSig/witness spend template |
| 05 | `lab05_compatibility.rs` | Sender-capability model across Base58Check/Bech32/Bech32m, best-format selection |
| 06 | `lab06_weight_fees.rs` | BIP141 weight (`stripped*3 + total`), virtual size (`ceil(weight/4)`), overflow-safe fee math |
| 07 | `lab07_bip39.rs` | Mnemonic/checksum validation, entropy/checksum bit accounting, seed derivation, passphrase comparison |
| 08 | `lab08_bip32.rs` | Master xpriv from seed, xpriv→xpub derivation, watch-only public-child derivation, hardened-step detection |
| 09 | `lab09_bip44.rs` | `m/purpose'/coin'/account'/change/index` decoding, human-readable description, index replacement, address derivation |
| 10 | `lab10_recovery.rs` | Format-parameterized derivation (P2PKH/P2SH-P2WPKH/P2WPKH/P2TR), joint BIP44+49+84 address set, repeatability proofs |

All implementations use `rust-bitcoin 0.32`'s real BIP32 (`Xpriv`/`Xpub`/`DerivationPath`)
and address (`Address::p2pkh`/`p2sh`/`p2wpkh`/`p2shwpkh`/`p2tr`) APIs plus the `bip39`
crate's `Mnemonic` — no cryptography was hand-rolled.

### 3. Added a runnable evidence generator

Created `examples/labs_demo.rs`, which exercises every one of the 40 lab functions
against the published public test mnemonic and fixed, disposable `[byte; 32]` demo
secret keys, and prints the real results. Run with:

```bash
cargo run --example labs_demo
```

This is the source of the concrete addresses/hashes/scripts/xprvs quoted in the
`submissions/` evidence files — nothing in this assignment used a real key or mnemonic.

### 4. Completed all 10 evidence submissions (`submissions/lab_01.md` .. `lab_10.md`)

Each file follows the required template (`## Commands used`, `## Terminal output`,
`## Evidence references`, `## Explanation`) with:
- The exact `cargo test --test lab_XX -- --nocapture` and `cargo run --example
  labs_demo` commands run.
- Real captured terminal output (test results and derived values).
- Cross-references to source files, test files, and grading logs.
- An original, from-scratch written explanation answering that lab's conceptual
  question (e.g. why a prefix isn't proof of validity, why hardened derivation can't
  be reversed from an xpub, why SegWit's discount isn't a flat whole-tx discount).

### 5. Verification

```bash
cargo test                                            # 40/40 tests passing
cargo fmt --check                                      # clean
cargo clippy --all-targets --all-features -- -D warnings   # clean, zero warnings
bash grader/grade.sh                                    # 70/70 automated points
```

`grader/grade.sh` (the same script GitHub Actions runs) reports, per lab: 4/4 public
tests passing and all 3 evidence sections complete — **70/70** automated total, ready
for the instructor's 0–30 explanation review.

### 6. Git workflow

- Fetched the upstream scaffold and implemented the labs on branch `rfb_labs_week_5`.
- Continued the same commit on branch `rust-for-bitcoin-5.0` (this fork's per-week
  branch naming convention), which now contains that work.
- Commit: `feat(week-5): implement Bitcoin address format and HD wallet labs`.

## Files created

```
rfb_labs_week_5/
├── .github/workflows/grade-week-5.yml   (fetched, unmodified)
├── Cargo.toml / Cargo.lock              (fetched, unmodified)
├── LABS.md / README.md                  (fetched, unmodified)
├── WORK_DONE.md                         (this file)
├── examples/
│   └── labs_demo.rs                     (new — evidence generator)
├── grader/
│   ├── check_evidence.sh                (fetched, unmodified)
│   └── grade.sh                         (fetched, unmodified)
├── src/
│   ├── error.rs / lib.rs / model.rs     (fetched, unmodified)
│   └── labs/
│       ├── lab01_addresses.rs           (implemented)
│       ├── lab02_p2pkh.rs               (implemented)
│       ├── lab03_p2sh.rs                (implemented)
│       ├── lab04_p2wpkh.rs              (implemented)
│       ├── lab05_compatibility.rs       (implemented)
│       ├── lab06_weight_fees.rs         (implemented)
│       ├── lab07_bip39.rs               (implemented)
│       ├── lab08_bip32.rs               (implemented)
│       ├── lab09_bip44.rs               (implemented)
│       └── lab10_recovery.rs            (implemented)
├── submissions/
│   └── lab_01.md .. lab_10.md           (completed evidence write-ups)
└── tests/
    └── lab_01.rs .. lab_10.rs           (fetched, unmodified — the public grading suite)
```

## Build status

- Compiles cleanly against `bitcoin = "0.32"` and `bip39 = "2.2"`.
- All 40 public tests pass.
- `cargo fmt --check` and `cargo clippy -- -D warnings` are both clean.
- Local automated grading score: **70/70**.

## Next steps

- Push `rust-for-bitcoin-5.0` and open the pull request per the top-level README's
  contribution workflow (fork → branch → commit → push → PR).
- Await the instructor's manual review of the `## Explanation` sections (0–30 points).
