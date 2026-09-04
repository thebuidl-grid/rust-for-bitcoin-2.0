# Lab 05 — Sender compatibility map

## Commands used

```bash
cargo test --test lab_05 -- --nocapture
```

## Terminal output

```
running 4 tests
test builds_the_four_format_map ... ok
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok
test names_the_required_human_encoding ... ok
test selects_the_most_modern_supported_format ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Evidence references

`src/labs/lab05_compatibility.rs`:

- `can_send_to` maps each `AddressFormat` to the one `SenderCapabilities` flag it needs:
  P2PKH/P2SH require Base58Check decoding, P2WPKH requires Bech32, P2TR requires
  Bech32m.
- `compatibility_report` builds the full four-format map for a wallet's capabilities.
- `best_supported_format` walks Taproot → P2WPKH → P2SH → P2PKH and returns the first
  one the wallet supports, i.e. prefers the newest format the sender can actually use.
- `required_encoding` names the exact encoding standard each format is locked to.

The "p2sh-era wallet" fixture in the tests (`base58_p2pkh: true, base58_p2sh: true,
bech32: false, bech32m: false`) correctly sends to P2PKH/P2SH but is rejected for
P2WPKH/P2TR until `bech32`/`bech32m` are turned on.

## Explanation

`3...` still decodes with Base58Check, same as `1...`, just a different version byte —
no new logic needed, so any wallet from the P2SH era (2012+) can build that output
blind, without knowing anything about what's hidden inside the redeemScript.

`bc1q...` is a different encoding altogether — Bech32, from BIP173, introduced years
later specifically for SegWit. A wallet that predates that BIP has no Bech32 decoder
at all. It's not that it rejects the address on purpose; it literally can't parse the
string.

Sending and spending support aren't the same capability, and this is where that
matters: an old wallet's inability to send to `bc1q...` is a gap in its own encoding
logic. It says nothing about whether SegWit outputs are actually spendable — any
SegWit-aware node validates and spends those UTXOs fine. Building an output that pays
a script family, and recognizing/unlocking a UTXO already locked to that family, are
different code paths. A wallet can be behind on one and fully caught up on the other.
