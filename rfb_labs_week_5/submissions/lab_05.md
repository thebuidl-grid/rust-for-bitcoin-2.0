# Lab 05 — Address compatibility map

## Commands used

TODO: List the Rust commands you ran.

## Terminal output

TODO: Record the four-format compatibility report.

## Evidence references

TODO: Link screenshots or describe attached evidence.

## Explanation

Sending compatibility depends on whether the sender can decode and validate the
recipient's address encoding. P2PKH and P2SH use Base58Check, native P2WPKH uses
Bech32, and Taproot P2TR uses Bech32m. A P2SH-era wallet may understand a `3...`
Base58Check address but have no Bech32 decoder, causing it to reject a `bc1q...`
native SegWit address even though both are valid Bitcoin destinations.

P2SH-P2WPKH provides a compatibility bridge. The recipient wraps a version-0
P2WPKH witness program inside P2SH, so an older sender only needs to construct a
normal P2SH output. The recipient later spends it using witness data. Sending and
spending support are therefore different: a sender only needs to understand the
destination address and create its scriptPubKey, while the owner of the output must
understand and satisfy the underlying spending rules. For a wallet that supports
several formats, the preferred single-key order is P2TR, P2WPKH, P2SH-P2WPKH, and
finally legacy P2PKH.
