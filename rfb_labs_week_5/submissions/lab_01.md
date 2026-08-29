# Lab 01 — Address and network identification

**Author:** [Christopher Dominic Eze](https://github.com/Christopherdominic)

## Commands used

```bash
cargo test --test lab_01
```

I also wrote a throwaway example (`examples/evidence.rs`) that calls
`inspect_address` and `expected_prefix` directly and prints what they return, so I
could see real values instead of just a pass/fail line:

```bash
cargo run --example evidence
```

I deleted that example file once I'd copied the output below — it isn't part of the
actual lab code, just a way to look at the results.

## Terminal output

```
running 4 tests
test identifies_human_readable_prefixes ... ok
test maps_regtest_prefixes ... ok
test rejects_an_address_for_the_wrong_network ... ok
test inspects_a_network_checked_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

From the example run, for a regtest P2PKH address built from a disposable test key:

```
address:       mrcNu71ztWjAQA6ww9kHiW3zBWSQidHXTQ
network:       regtest
format:        P2pkh
scriptPubKey:  76a91479b000887626b294a914501a4cd226b58b23598388ac
expected P2WPKH regtest prefix: Some("bcrt1q")
```

## Evidence references

- `src/labs/lab01_addresses.rs` — `identify_prefix`, `expected_prefix`,
  `inspect_address`, `script_pubkey_hex`.
- `tests/lab_01.rs` — all four tests above, including
  `rejects_an_address_for_the_wrong_network`, which feeds the same regtest address
  into `inspect_address(..., Network::Bitcoin)` and checks it comes back `Err`.
- The scriptPubKey above (`76a914...88ac`) decodes to
  `OP_DUP OP_HASH160 <20-byte-hash> OP_EQUALVERIFY OP_CHECKSIG`, i.e. P2PKH, which
  matches the `m`/`n` prefix the address starts with.
- Poking at `identify_prefix` past the public test cases: my first version found the
  bech32 separator with `address.rfind('1')`, which is wrong — `1` is a valid
  Base58 character, so a legacy testnet address like `m1qzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz`
  can contain a `1` followed by `q`/`p` purely by coincidence and get misread as
  P2WPKH/P2TR. Fixed it to match on the known HRPs (`bc1`, `tb1`, `bcrt1`) instead of
  the last `1` in the string. Good reminder that "prefix is only a clue" cuts both
  ways — even my own prefix-sniffing code can misread a prefix if it's not careful.

## Explanation

The prefix on an address is really just the first thing a human (or a wallet) sees,
and it comes from whatever byte(s) Base58Check or bech32 put at the front once the
payload is encoded. For legacy formats that's a version byte baked into the
Base58Check encoding — `0x00` gives you a `1...` mainnet P2PKH address, `0x6f` gives
you the `m`/`n` regtest/testnet one, `0x05`/`0xc4` do the same for P2SH's `3`/`2`. For
bech32/bech32m it's the human-readable part (`bc`, `tb`, `bcrt`) plus the witness
version encoded in the first data character (`q` = version 0, `p` = version 1).

So a prefix tells you two things at once: which network the address claims to be for,
and which script type it claims to unlock. But it's a claim, not a proof. Base58Check
addresses carry a 4-byte checksum (double-SHA256 of the payload, first four bytes) —
if you flip one character while copying an address, decoding it will almost always
fail the checksum, which is exactly the point: it catches transcription errors, it's
not a security mechanism against a malicious sender. Bech32/bech32m has its own
BCH-code-based checksum doing the same job, and it also happens to be much better at
catching multi-character errors than Base58Check is.

Network is a separate check on top of that. Nothing stops someone from generating a
perfectly valid, checksum-correct testnet address and handing it to you expecting a
mainnet payment (or vice versa) — the prefix just tells you what the encoder claims,
your code still has to decide whether that matches the network you're actually
operating on. That's why `inspect_address` doesn't stop at parsing; it calls
`require_network` and returns an error if the address was built for a different
chain, even though the address is otherwise completely well-formed. `rust-bitcoin`
enforces this by making `Address::from_str` hand you back a
`Address<NetworkUnchecked>` — you can't get the scriptPubKey out of it at all until
you've explicitly told it which network you expect and it's agreed the address
matches.

In short: prefix is a fast, human-readable hint; checksum verification and explicit
network checking are what actually make it safe to send to.
