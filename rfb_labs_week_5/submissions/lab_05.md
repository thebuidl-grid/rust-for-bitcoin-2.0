# Lab 05 — Address compatibility map

## Commands used

```bash
cargo test --test lab_05
cargo fmt --check
bash grader/grade.sh
```

## Terminal output

```text
running 4 tests
test names_the_required_human_encoding ... ok
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok
test builds_the_four_format_map ... ok
test selects_the_most_modern_supported_format ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Unit tests in `tests/lab_05.rs` verifying sender compatibility rules across Base58Check (P2PKH, P2SH), Bech32 (P2WPKH), and Bech32m (P2TR), as well as fallback and format preference evaluation.

## Explanation

Address compatibility is determined by the string parsing and encoding libraries implemented in the sender's wallet software. An older P2SH-era wallet supports Base58Check encoding, allowing it to parse addresses starting with `3...` (or `2...` on testnet/regtest) and pay them as standard P2SH outputs without needing to understand that the inner script wraps a SegWit witness program. However, native SegWit (`bc1q...`) and Taproot (`bc1p...`) use Bech32 (BIP173) and Bech32m (BIP350) encodings with distinct character sets and checksum polynomial algorithms. An legacy wallet lacking Bech32/Bech32m decoders will treat `bc1q...` string inputs as malformed addresses and refuse to generate payments to them.
