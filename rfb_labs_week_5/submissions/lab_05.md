# Lab 05 — Address compatibility map

## Commands used

```bash
cargo test --test lab_05
bash grader/grade.sh
```

## Terminal output

running 4 tests

test builds_the_four_format_map ... ok

test selects_the_most_modern_supported_format ... ok

test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok

test names_the_required_human_encoding ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

## Evidence references

All four public tests in `tests/lab_05.rs` pass, covering:
- A P2SH-era wallet (Base58Check P2PKH/P2SH only) correctly accepting P2SH
  addresses and rejecting native SegWit (`older_p2sh_wallet_accepts_wrapped_but_not_native`)
- The full four-format compatibility report for that same wallet
  (`builds_the_four_format_map`)
- Selecting the most modern format a wallet supports as capabilities are
  added incrementally (`selects_the_most_modern_supported_format`)
- Naming the correct encoding requirement — Base58Check, Bech32, or
  Bech32m — for each format (`names_the_required_human_encoding`)
## Explanation


A P2SH-era wallet accepts `3...` addresses and rejects `bc1q...` addresses
because the two use entirely different text encodings, not just different
prefixes.

`1...` and `3...` addresses are both encoded with Base58Check — the same
alphabet, checksum algorithm, and decoding logic, differing only in a
single version byte that Base58Check decodes and checks. A wallet that
already knows how to decode Base58Check for P2PKH gets P2SH support
essentially "for free," since it's the identical encoding scheme with one
different version byte — no new decoder is needed, which is exactly why
P2SH-wrapped SegWit (`3...`) was designed the way it was: it lets older,
Base58Check-only wallets pay into SegWit outputs without ever needing to
understand Bech32 at all.

`bc1q...` addresses use Bech32, a completely different encoding — different
character set, different checksum algorithm, no version byte in the same
sense. A wallet that only implements a Base58Check decoder has no code path
at all for parsing Bech32 text; it isn't simply an "unsupported version," it
is unrecognizable data. That's why sending support genuinely depends on
which encodings a wallet has implemented, not just which script types it
recognizes semantically.

This is also why sending support differs from spending support: a wallet
only needs to *encode/decode an address string* correctly to send to it —
it never needs to construct or understand the witness data required to
*spend* from that same address. A P2SH-era wallet could send funds to a
`bc1q...` Taproot address once it learns Bech32/Bech32m encoding, without
ever being able to construct a Taproot signature to spend those same funds
back out itself.

