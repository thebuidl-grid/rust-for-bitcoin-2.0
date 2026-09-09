# Lab 01 — Address and network identification

## Commands used

```bash
cargo test --test lab_01
bash grader/grade.sh
```


## Terminal output

running 4 tests

test identifies_human_readable_prefixes ... ok

test maps_regtest_prefixes ... ok

test rejects_an_address_for_the_wrong_network ... ok

test inspects_a_network_checked_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

## Evidence references

All four public tests in `tests/lab_01.rs` pass, covering:
- Prefix identification for P2PKH, P2SH, P2WPKH, and P2TR addresses
  (`identifies_human_readable_prefixes`)
- Expected prefix mapping for every format on regtest
  (`maps_regtest_prefixes`)
- Full address inspection — parsing, network check, format detection, and
  scriptPubKey — against a locally generated regtest P2PKH address
  (`inspects_a_network_checked_address`)
- Rejection of a valid regtest address when checked against the wrong
  network (`rejects_an_address_for_the_wrong_network`)

## Explanation

`identify_prefix` only inspects the first few characters of a string — it
doesn't parse, checksum-validate, or network-check anything. A malformed or
corrupted address could still happen to start with `1` or `bc1q` and be
misidentified as a valid P2PKH or P2WPKH address by prefix alone, when it's
actually garbage data with a typo or a flipped bit.

`inspect_address` shows the real validation path: `Address::from_str` first
performs full Base58Check or Bech32(m) checksum verification, rejecting
anything with corrupted data even if the prefix looked right. Then
`require_network` separately verifies the address was actually intended for
the target network — a well-formed, checksum-valid mainnet address could
still be rejected if the caller expected regtest, since sending funds to an
address on the wrong network can mean permanent loss.

So prefix inspection is a fast, useful hint for a UI or a first-pass filter,
but it provides no cryptographic guarantee of validity — only full parsing
(checksum) plus an explicit network check together confirm an address is
genuinely safe to use.

