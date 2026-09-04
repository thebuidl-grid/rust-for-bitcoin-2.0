# Lab 05 — Address compatibility across generations

## Commands used

```
cargo test --test lab_05 -- --nocapture
cargo fmt --check
```

Implementation: `src/labs/lab05_compatibility.rs` — `can_send_to`,
`compatibility_report`, `best_supported_format`, `required_encoding`.

## Terminal output

```
running 4 tests
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok
test selects_the_most_modern_supported_format ... ok
test names_the_required_human_encoding ... ok
test builds_the_four_format_map ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Observed behaviour for a "P2SH-era" wallet (`base58_p2pkh`, `base58_p2sh` = true;
`bech32`, `bech32m` = false):
- `can_send_to(P2sh)` = true, `can_send_to(P2wpkh)` = false.
- `compatibility_report` = `{ p2pkh: true, p2sh_p2wpkh: true, p2wpkh: false, p2tr: false }`.
- `best_supported_format` = `P2sh`; enabling `bech32` promotes it to `P2wpkh`;
  enabling `bech32m` promotes it to `P2tr`.
- `required_encoding`: P2PKH/P2SH = `Base58Check`, P2WPKH = `Bech32`, P2TR = `Bech32m`.

## Evidence references

- `src/labs/lab05_compatibility.rs` — single `can_send_to` match reused by the report.
- `tests/lab_05.rs` — the capability struct and expected `CompatibilityReport`.
- Terminal block above from `cargo test --test lab_05`.

## Explanation

Whether a sender can pay you depends only on whether their wallet can *decode the
address string*, not on anything about your keys. A P2SH-era wallet knows
Base58Check, so it can pay `1...` (P2PKH) and `3...` (P2SH). A wrapped SegWit
receive address (P2SH-P2WPKH) is still a plain `3...` P2SH address on the outside,
so that same old wallet pays it fine — the witness details are hidden behind the
script hash. A *native* SegWit address `bc1q...` is bech32, and Taproot `bc1p...`
is bech32m (a checksum-constant change specifically to separate v1+ from v0). A
wallet that never implemented bech32/bech32m simply cannot parse those strings and
will refuse to send. So the ordering of "best" formats — Taproot > native SegWit >
wrapped SegWit > legacy — is really an ordering of decoder requirements, and
wrapped SegWit exists as the bridge for senders stuck on Base58.
