# Lab 10 — Deterministic recovery across BIP44/49/84

## Commands used

```bash
cargo test --test lab_10 -- --nocapture
```

## Terminal output

```
running 4 tests
test changing_only_the_index_changes_the_address ... ok
test identical_recovery_inputs_repeat ... ok
test format_selection_changes_the_lock_target ... ok
test derives_three_regtest_address_families ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
```

## Evidence references

All four public tests in `tests/lab_10.rs` pass against `src/labs/lab10_recovery.rs`, all on
regtest with the public test mnemonic: `derive_address_set(account=0, index=0)` derives index 0 on
all three branches at once — `m/44'/1'/0'/0/0` (P2PKH, `m`/`n` prefix), `m/49'/1'/0'/0/0` (wrapped
P2WPKH via `Address::p2shwpkh`, `2` prefix), and `m/84'/1'/0'/0/0` (native P2WPKH, `bcrt1q`
prefix); `recovery_is_repeatable` shows the identical mnemonic/passphrase/path/network reproduces
the identical address on two separate derivations; `changing_index_changes_address` shows that
changing only the final index (`.../0` → `.../1`) is enough to select a different address; and
`format_selection_changes_the_lock_target` shows the same path derives a different address
depending on whether it's locked as P2PKH vs. P2WPKH.

## Explanation

**Why identical recovery inputs reproduce the same address:** every step of this derivation chain
is a pure, deterministic function of its inputs — `Mnemonic::to_seed(passphrase)` is deterministic
given the words and passphrase; `Xpriv::new_master` is a deterministic HMAC-SHA512 of that seed;
and each `derive_priv` step down a path is a deterministic HMAC-SHA512 of the parent's key,
chain code, and child index. There is no randomness and no external state anywhere in this chain.
So `recovery_is_repeatable` deriving the exact same mnemonic + passphrase + path + network twice
is guaranteed, by construction, to produce the same private key and therefore the same address
both times — this determinism is the entire point of BIP32/39/44: it's what makes a wallet
"recoverable" from words alone, rather than needing to store every generated key individually.

**Why restoring a wallet depends on more than the mnemonic:** `format_selection_changes_the_lock_target`
proves that the *same derived key* produces a *different address* depending on which script type
it's locked into (P2PKH vs. P2WPKH here — the same principle extends to P2SH-wrapped SegWit and
Taproot). That means the mnemonic and passphrase alone are not sufficient to restore a wallet's
funds-visibility — you also need to know the *derivation paths* and *script/address types* the
original wallet used (BIP44 vs. BIP49 vs. BIP84, which account and index range, receive vs.
change), because a different combination of those conventions derives a completely different set
of addresses from the exact same root key. This is why real wallets standardize on specific
purpose values (44'/49'/84'/86') per script type — it lets a *different* wallet application,
given only the same words, know which addresses to scan for balance during recovery, without the
user having to separately remember or transcribe path conventions.
