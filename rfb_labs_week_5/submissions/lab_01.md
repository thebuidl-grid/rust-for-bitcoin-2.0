# Lab 01 — Address and network identification

## Commands used

```bash
cargo test --test lab_01 -- --nocapture
cargo fmt --check
cargo clippy --all-targets
```

Implementation lives in `src/labs/lab01_addresses.rs`: `identify_prefix`,
`expected_prefix`, `inspect_address`, and `script_pubkey_hex`.

## Terminal output

```
running 4 tests
test identifies_human_readable_prefixes ... ok
test maps_regtest_prefixes ... ok
test inspects_a_network_checked_address ... ok
test rejects_an_address_for_the_wrong_network ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo fmt --check` and `cargo clippy --all-targets` both completed with no
diffs and no warnings.

## Evidence references

- `src/labs/lab01_addresses.rs` — the four implemented functions.
- `tests/lab_01.rs` — the public suite exercised above, all 4 tests green.
- `bash grader/grade.sh` reported `01 | 4/4 | 4 | ... |` in
  `grading/score.md` for this lab's execution row.

## Explanation

The human-readable prefix (`1`, `3`, `bc1q`, `bc1p`, ...) is only the Base58Check
version byte or Bech32/Bech32m human-readable part rendered back to text — it
tells a wallet which script family an address probably encodes, nothing more.
`identify_prefix` in this lab does exactly that lightweight, string-only
inspection, and the test feeds it `"bc1pexample"`, a string that is not a
valid Taproot address at all, to make the point: prefix matching alone cannot
catch a truncated address, a single mistyped character, or bytes copied for
the wrong network.

That is why `inspect_address` and `script_pubkey_hex` do real work instead of
trusting the prefix: they call `Address::from_str` to decode and checksum-verify
the Base58Check/Bech32 payload (catching typos and corruption the prefix can't
see), then call `.require_network(network)` to reject an address that decodes
fine but was minted for a different chain — e.g. a mainnet `1...` address
checksums correctly yet must never be paid on regtest. The test
`rejects_an_address_for_the_wrong_network` proves this: the same address
string that passes on `Regtest` is rejected on `Bitcoin` even though its
checksum is valid. Prefix, checksum, and network are three independent checks;
skipping any one of them lets an invalid or wrong-network destination through.
