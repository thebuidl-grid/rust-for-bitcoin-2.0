# Lab 05 — Sender compatibility map

## Commands used

```bash
cargo test --test lab_05
cargo run -- 5
```

The runner builds four sender profiles, one per address generation, and reports what
each can pay, which format each should be given, and the encoding every format
needs.

## Terminal output

```text
$ cargo test --test lab_05
running 4 tests
test builds_the_four_format_map ... ok
test names_the_required_human_encoding ... ok
test selects_the_most_modern_supported_format ... ok
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo run -- 5
== Lab 05: Sender compatibility ==
  2013 P2PKH-only wallet
    P2PKH true | P2SH-P2WPKH false | P2WPKH false | P2TR false
    best supported Some(P2pkh)
  2017 P2SH-era wallet
    P2PKH true | P2SH-P2WPKH true | P2WPKH false | P2TR false
    best supported Some(P2sh)
  2019 Bech32 wallet
    P2PKH true | P2SH-P2WPKH true | P2WPKH true | P2TR false
    best supported Some(P2wpkh)
  2023 Bech32m wallet
    P2PKH true | P2SH-P2WPKH true | P2WPKH true | P2TR true
    best supported Some(P2tr)
  P2pkh requires Base58Check
  P2sh requires Base58Check
  P2wpkh requires Bech32
  P2tr requires Bech32m
```

## Evidence references

Implementation in `src/labs/lab05_compatibility.rs`, tests in `tests/lab_05.rs`, runner
in `src/main.rs` under `lab05`.

The four rows are a timeline. The P2PKH-only profile pays `1...` and nothing else. The
P2SH-era profile gains `3...`, and with it wrapped SegWit, without knowing what SegWit
is. The bech32 profile gains `bc1q...`. Only the bech32m profile can pay `bc1p...`.

best_supported_format walks down that list one generation at a time, so the same call
returns P2sh, then P2wpkh, then P2tr as capabilities are switched on. The encoding
table underneath explains it: two formats share Base58Check, and the two newer ones
need two different bech32 variants.

## Explanation

What a sender can pay depends on the address encodings it can decode. It does not depend
on which scripts it understands.

An older wallet accepts `3...` because Base58Check P2SH support arrived in 2012 with
BIP16, years before SegWit. Paying a wrapped SegWit address, that wallet decodes a
version byte and a 20-byte hash, builds OP_HASH160 <hash> OP_EQUAL, and stops. It never
learns the hash preimage is a witness program. BIP49 works precisely because it hides
new behaviour behind an output format old software already builds.

The same wallet rejects `bc1q...`. Bech32 is a different encoding from BIP173, with no
version byte to look up and no double SHA256 checksum to verify, so a Base58Check
decoder has nothing to work with and refuses. BIP350 repeats this one level up. Taproot
addresses use bech32m, which differs from bech32 only in the checksum constant, so a
bech32-only decoder rejects `bc1p...` too. Both refusals are the design working. A
guess would burn funds.

Sending and spending are separate questions because they involve different parties.
Sending needs only the ability to turn an address string into a scriptPubKey, and the
sender signs nothing belonging to the receiver. Spending needs a witness or ScriptSig
that satisfies the script, the correct sighash, and for SegWit inputs the input amount
that BIP143 requires. A wallet can therefore pay a `bc1q` address while being unable to
hold one. The reverse happens too: a watch-only setup can track outputs it will never
spend.

For a receiver the practical effect is that address choice is a negotiation with the
least capable sender they expect. best_supported_format encodes the sensible default,
which is to take the cheapest modern format the sender can pay and step back one
generation at a time instead of defaulting everyone to legacy.
