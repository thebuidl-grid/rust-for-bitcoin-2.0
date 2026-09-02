# Lab 05 — Address compatibility map

## Commands used

```shell
test@pop-os:~/Desktop/rust/rust-for-bitcoin-2.0/rfb_labs_week_5$ cargo test --test lab_05
```

## Terminal output

```terminaloutput
running 4 tests
test builds_the_four_format_map ... ok
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok
test selects_the_most_modern_supported_format ... ok
test names_the_required_human_encoding ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

For a P2SH-era wallet (`base58_p2pkh: true`, `base58_p2sh: true`, `bech32: false`,
`bech32m: false`) the compatibility report is:

| Format | Supported |
|--------|-----------|
| P2PKH (`1...`) | yes |
| P2SH / wrapped SegWit (`3...`) | yes |
| Native P2WPKH (`bc1q...`) | no |
| Taproot (`bc1p...`) | no |

`can_send_to` returns true only for `P2sh` and `P2pkh`; `best_supported_format`
selects `P2sh`.

## Evidence references

```
Code: src/labs/lab05_compatibility.rs
Test: tests/lab_05.rs
```

## Explanation

Sender compatibility is decided by which **human-readable encoding** the wallet can
decode and re-encode when it pays an output. Legacy P2PKH and P2SH use Base58Check;
native SegWit v0 (P2WPKH) uses Bech32; Taproot (P2TR) uses Bech32m.

A P2SH-era wallet typically implemented Base58Check and predates native SegWit
support, so it can pay `1...` (P2PKH) and `3...` (P2SH). Wrapped SegWit (P2SH-P2WPKH)
also works because from the sender's perspective it is just a `3...` P2SH address —
the witness program is tucked inside the P2SH wrapper, so the sender never needs
Bech32. That is why an older wallet accepts `3...` but rejects `bc1q...`: paying a
native SegWit output requires Bech32 decoding, which the wallet does not implement.

Sending support differs from **spending** support. Sending only needs to decode the
recipient's address and build a scriptPubKey for it; spending (moving those coins
later) additionally requires the wallet to produce valid signatures and, for SegWit
outputs, to commit to a specific transaction digest and witness. A wallet may be able
to send to a `bc1q...` output yet still be unable to spend from it, and vice versa.
