# Lab 05 — Compatibility map

## Commands used

```bash
cargo test --test lab_05 -- --nocapture
cargo fmt --check
cargo clippy --all-targets
```

Implementation lives in `src/labs/lab05_compatibility.rs`: `can_send_to`,
`compatibility_report`, `best_supported_format`, and `required_encoding`.

## Terminal output

```
running 4 tests
test builds_the_four_format_map ... ok
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok
test names_the_required_human_encoding ... ok
test selects_the_most_modern_supported_format ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

- `src/labs/lab05_compatibility.rs` — the capability-to-format map and the
  Taproot > native SegWit > wrapped SegWit > legacy preference order.
- `tests/lab_05.rs` — `older_p2sh_wallet_accepts_wrapped_but_not_native`
  models a `base58_p2pkh + base58_p2sh` wallet (no `bech32`/`bech32m`) and
  confirms it can send to `P2sh` but not `P2wpkh`;
  `selects_the_most_modern_supported_format` walks that wallet through
  gaining `bech32` then `bech32m` and checks the preferred format upgrades
  from `P2sh` to `P2wpkh` to `P2tr` in step.
- `bash grader/grade.sh` recorded `05 | 4/4 | 4 | ...` for this lab.

## Explanation

A pre-SegWit wallet's sending code only knows how to build a Base58Check
decoder and a legacy `scriptSig` — it recognizes the `1`/`3` (or `m/n`/`2` on
test networks) prefixes and produces `OP_DUP OP_HASH160 ... OP_CHECKSIG` or
`OP_HASH160 ... OP_EQUAL` locks. `3...` addresses (P2SH) predate SegWit by
years, so that wallet accepts them without any code change. A `bc1q...`
address, however, uses Bech32 encoding and, once decoded, produces a witness
program the legacy wallet's transaction builder has no path for — it isn't
that the wallet refuses on principle, it literally cannot construct a valid
output for a script format it never learned. `required_encoding` captures the
same split: P2PKH/P2SH need Base58Check, P2WPKH needs Bech32, and P2TR needs
Bech32m — three different parsers, and a wallet only supports the formats
whose parser and script builder it ships with, as modeled by
`SenderCapabilities`.

Sending support and spending support are genuinely different capabilities
because they run on different machines at different times. Sending requires
only the ability to *build* an output — decode the address, embed the right
locking script, and broadcast. Spending later requires the *receiving*
wallet to construct a valid witness or scriptSig against that exact locking
script, which needs its own key-management and signing support for that
script type. A wallet can therefore send to a script type it could never
spend from (e.g. an exchange sending to a novel address format a customer's
wallet already understands), and, more relevantly here, an old wallet's
inability to send to `bc1q...` says nothing about whether some other, newer
wallet holding the matching private key could spend a `bc1q...` output just
fine.
