# Lab 10 — Deterministic recovery across address families

## Commands used

```bash
cargo test --test lab_10
cargo fmt --check
```

Only the public class mnemonic on `Network::Regtest` was used — all addresses below are
disposable test-family output, never real recovery material.

## Terminal output

```text
$ cargo test --test lab_10
running 4 tests
test identical_recovery_inputs_repeat ... ok
test changing_only_the_index_changes_the_address ... ok
test format_selection_changes_the_lock_target ... ok
test derives_three_regtest_address_families ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

`cargo fmt --check` produced no diff.

## Evidence references

- Implementation: `src/labs/lab10_recovery.rs`
- Test suite: `tests/lab_10.rs`
- `derive_address_set(MNEMONIC, "", 0, 0, Regtest)` derives index 0 on all three branches at once:
  `bip44_p2pkh` starts with `m`/`n` (regtest legacy prefix), `bip49_p2sh_p2wpkh` starts with `2`
  (regtest P2SH prefix, since BIP49 wraps SegWit in P2SH), and `bip84_p2wpkh` starts with `bcrt1q`
  (regtest native SegWit prefix) — confirming `derives_three_regtest_address_families`.
- `recovery_is_repeatable` derives `m/84'/1'/0'/0/0` twice from the same mnemonic/passphrase and
  gets the identical address both times.
- `changing_index_changes_address` derives `.../0/0` and `.../0/1` and confirms the addresses
  differ.
- `derive_address_for_path` with the same path but `AddressFormat::P2pkh` vs. `AddressFormat::P2wpkh`
  produces two different addresses, since the format choice changes the *locking script*, not the
  underlying key.

## Explanation

Identical recovery inputs reproduce the same address because every step from seed to address is a
pure function with no external state or randomness: `PBKDF2(mnemonic, passphrase)` deterministically
produces the seed, BIP32's HMAC-SHA512-based child derivation deterministically walks the same
path to the same key every time, and encoding that key as an address is a pure, table-driven
transformation. There is nothing probabilistic anywhere in that chain, which is exactly what
`recovery_is_repeatable` demonstrates by calling the same derivation twice and getting a byte-for-
byte-identical result — and what `changing_index_changes_address` demonstrates from the other
side: since the chain code and index both feed the HMAC that produces a child key, changing only
the index necessarily changes the derived key and therefore the address.

But "restoring a wallet" is not just "the mnemonic reproduces a key" — it also depends on
conventions the mnemonic alone cannot encode. The same seed produces a different address depending
on which BIP44/49/84 path is walked (different `purpose'` and `coin_type'` levels) and which
script format wraps the resulting key (P2PKH vs. P2SH-wrapped-P2WPKH vs. native P2WPKH), as shown
by `derive_address_for_path` returning different addresses for the same path with different
`format` arguments. A wallet that restores a mnemonic but guesses the wrong path/script convention
will derive real, validly-signable keys — just not the ones holding the funds, because those funds
were sent to addresses generated under a specific path+script convention that has to be known (or
guessed correctly, as many recovery tools do by scanning several standard combinations) alongside
the mnemonic and passphrase themselves.

