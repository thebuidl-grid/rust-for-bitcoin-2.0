# Lab 05 — Address compatibility map

## Commands used

```bash
cargo test --test lab_05
cargo fmt --check
```

## Terminal output

```text
$ cargo test --test lab_05
running 4 tests
test builds_the_four_format_map ... ok
test names_the_required_human_encoding ... ok
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok
test selects_the_most_modern_supported_format ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo fmt --check` produced no diff.

## Evidence references

- Implementation: `src/labs/lab05_compatibility.rs`
- Test suite: `tests/lab_05.rs`
- For a P2SH-era wallet (`base58_p2pkh: true, base58_p2sh: true, bech32: false, bech32m: false`),
  `compatibility_report` returns
  `CompatibilityReport { p2pkh: true, p2sh_p2wpkh: true, p2wpkh: false, p2tr: false }`.
- `best_supported_format` walks Taproot → native SegWit → wrapped SegWit → legacy as capabilities
  are added one at a time, confirmed by `selects_the_most_modern_supported_format`.
- `required_encoding` maps P2PKH/P2SH to `"Base58Check"`, P2WPKH to `"Bech32"`, and P2TR to
  `"Bech32m"`.

## Explanation

An older, P2SH-era wallet accepts `3...` and rejects `bc1q...` because those two address types
require different decoders entirely, and the wallet only shipped with one of them. A `3...`
address is Base58Check: version byte + 20-byte hash + 4-byte checksum, all decoded with the same
Base58 alphabet and SHA256d checksum logic the wallet already needs for `1...` addresses — no new
code path required, and critically, the wallet doesn't need to know or care that the redeemScript
behind that P2SH hash happens to be a wrapped SegWit program (`OP_0 <20-byte-hash>`). A `bc1q...`
address, by contrast, is Bech32 (BIP173), a completely different alphabet, checksum algorithm, and
parsing rule introduced alongside SegWit itself; a wallet written before BIP173 simply has no code
that recognizes the `bc1` human-readable part or the bech32 charset. This is also why *sending*
support and *spending* support differ: a wallet can be capable of paying a P2WPKH or P2SH-wrapped
output (sending — it just needs to build a normal output script) while still being unable to prove
ownership of one, or vice versa, since spending requires the wallet to construct the correct
witness/ScriptSig for that script type, a separate capability from parsing its address format.

