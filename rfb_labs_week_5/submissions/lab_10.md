# Lab 10 — Deterministic recovery across BIP44/49/84

## Commands used

```bash
cargo test --test lab_10 -- --nocapture
cargo fmt --check
cargo clippy --all-targets
```

Implementation lives in `src/labs/lab10_recovery.rs`: `derive_address_for_path`,
`derive_address_set`, `recovery_is_repeatable`, and
`changing_index_changes_address`.

## Terminal output

```
running 4 tests
test format_selection_changes_the_lock_target ... ok
test changing_only_the_index_changes_the_address ... ok
test identical_recovery_inputs_repeat ... ok
test derives_three_regtest_address_families ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

`derives_three_regtest_address_families` derives index 0 from one recovery
root on all three branches and shows the regtest prefixes / script families
side by side:

| Branch | Path prefix | Address prefix | Script family |
|---|---|---|---|
| BIP44 | `m/44'/1'/0'/0/0` | `m`/`n` | legacy P2PKH |
| BIP49 | `m/49'/1'/0'/0/0` | `2` | P2SH-wrapped P2WPKH |
| BIP84 | `m/84'/1'/0'/0/0` | `bcrt1q` | native P2WPKH |

## Evidence references

- `src/labs/lab10_recovery.rs` — one mnemonic/passphrase root driving three
  independent branch derivations and format selections.
- `tests/lab_10.rs` — `identical_recovery_inputs_repeat` derives
  `m/84'/1'/0'/0/0` twice from identical inputs and asserts equality;
  `changing_only_the_index_changes_the_address` derives index 0 and index 1
  on the same branch and asserts inequality; `format_selection_changes_the_lock_target`
  derives the same path `m/44'/1'/0'/0/0` as both P2PKH and P2WPKH and shows
  the two resulting addresses differ.
- `bash grader/grade.sh` recorded `10 | 4/4 | 4 | ...` for this lab.

## Explanation

BIP32/39 derivation is a pure function of its inputs: the mnemonic and
passphrase fix a 512-bit seed via PBKDF2, that seed fixes a master `xpriv`
and chain code via HMAC-SHA512, and each subsequent derivation step is itself
HMAC-SHA512 over the parent key, parent chain code, and child index — nowhere
in that chain is there randomness, wall-clock time, or any external state.
`recovery_is_repeatable` calls the exact same derivation twice from identical
mnemonic, passphrase, path, format, and network and gets byte-identical
addresses back, which is the whole point of "deterministic": the same
recovery inputs always retrace the same path through the same math to the
same key, which is what lets a wallet be recovered from words alone on any
compliant software, any time. `changing_index_changes_address` shows the
complementary fact — changing even the last path component changes the HMAC
input and therefore the derived key completely, so index 0 and index 1 are
unrelated-looking addresses even though they descend from the same account.

Reproducing an address, however, is not the same as reproducing a *wallet's
view of its funds*, because the mnemonic alone underdetermines the derivation
— it only fixes the seed. `derive_address_set` shows the same account/index
pair producing three unrelated addresses (`m`/`n`, `2`, `bcrt1q...`) purely by
changing the BIP44/49/84 purpose level, and
`format_selection_changes_the_lock_target` shows the same underlying path
producing different addresses depending only on which script type the caller
asks for at the end (P2PKH vs. P2WPKH from the very same derived key). A
recovery tool also has to know, and agree with the original wallet on, which
purpose/coin-type/account path convention and which script family to derive
at each address — restoring with the right words but the wrong path or script
convention silently walks a *different*, empty branch of the same tree
instead of failing, which is why wallets standardize on and record BIP44/49/84
account structure alongside the mnemonic rather than relying on the mnemonic
alone.
