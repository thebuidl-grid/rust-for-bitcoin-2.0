# Lab 05 — Address compatibility map

## Commands used

```bash
cargo test --test lab_05 -- --nocapture
cargo run --example labs_demo
```

## Terminal output

```text
$ cargo test --test lab_05 -- --nocapture
running 4 tests
test names_the_required_human_encoding ... ok
test builds_the_four_format_map ... ok
test selects_the_most_modern_supported_format ... ok
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
$ cargo run --example labs_demo   (Lab 05 section)
compatibility_report(p2sh_era) = CompatibilityReport { p2pkh: true, p2sh_p2wpkh: true, p2wpkh: false, p2tr: false }
best_supported_format(p2sh_era) = Some(P2sh)
compatibility_report(modern) = CompatibilityReport { p2pkh: true, p2sh_p2wpkh: true, p2wpkh: true, p2tr: true }
best_supported_format(modern) = Some(P2tr)
required_encoding(P2pkh) = "Base58Check"
required_encoding(P2sh) = "Base58Check"
required_encoding(P2wpkh) = "Bech32"
required_encoding(P2tr) = "Bech32m"
```

## Evidence references

- Implementation: [`src/labs/lab05_compatibility.rs`](../src/labs/lab05_compatibility.rs)
- Public test suite: [`tests/lab_05.rs`](../tests/lab_05.rs) — 4/4 passing, logged in
  [`grading/logs/lab_05.log`](../grading/logs/lab_05.log).
- `p2sh_era` wallet (`base58_p2pkh: true, base58_p2sh: true, bech32: false, bech32m:
  false`) is exactly the capability set of a pre-2017, pre-SegWit-adoption wallet.

## Explanation

A P2SH-era wallet accepts `3...` (or, on regtest/testnet, `2...`) addresses but rejects
`bc1q...` addresses purely because of **encoding**, not spending policy — the two
address kinds require completely different parsers:

- `3...`/`2...` addresses are **Base58Check**: version byte + 20-byte hash + 4-byte
  SHA256d checksum, all Base58-encoded. This encoding predates SegWit entirely, so any
  wallet capable of sending Bitcoin at all already speaks Base58Check — it is how both
  P2PKH and P2SH addresses have always been encoded. The wallet has no idea whether the
  hash inside a `3...` address commits to a simple script or a SegWit-wrapping
  redeemScript; from its point of view it is just building an ordinary P2SH output.
- `bc1q...` addresses are **Bech32** (BIP173), an entirely different, SegWit-specific
  encoding introduced alongside SegWit itself. A wallet's Base58Check decoder cannot
  parse Bech32 strings — it is not a variant or extension, it is a different alphabet,
  checksum algorithm, and human-readable-part scheme. Without an explicit Bech32
  decoder added to the wallet's code, `bc1q...` is simply unrecognized input.

This is why `can_send_to`/`compatibility_report` model capability per *encoding*
(`base58_p2pkh`, `base58_p2sh`, `bech32`, `bech32m`) rather than per spending-script
type: **sending support** is about whether the sender's wallet can parse and encode the
recipient's address string at all, which is a function of which encodings it
implements. **Spending support**, by contrast, is about whether a wallet *holding* a
given output can construct a valid witness/ScriptSig for it — a separate concern the
recipient's own wallet handles, unrelated to what encoding the sender used to address
the payment. A P2SH-wrapped SegWit address deliberately reuses the old Base58Check
encoding precisely so legacy senders can pay it without ever needing a Bech32 decoder,
which is exactly why `best_supported_format` prefers `P2sh` (wrapped SegWit) over raw
`P2pkh` once `base58_p2sh` is available, but still ranks native `P2wpkh`/`P2tr` higher
once Bech32/Bech32m support is added.
