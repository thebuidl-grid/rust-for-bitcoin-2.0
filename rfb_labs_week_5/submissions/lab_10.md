# Lab 10 — Deterministic recovery across address families

## Commands used

```bash
cargo test --test lab_10 -- --nocapture
cargo run --example labs_demo
```

## Terminal output

```text
$ cargo test --test lab_10 -- --nocapture
running 4 tests
test changing_only_the_index_changes_the_address ... ok
test format_selection_changes_the_lock_target ... ok
test identical_recovery_inputs_repeat ... ok
test derives_three_regtest_address_families ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

```text
$ cargo run --example labs_demo   (Lab 10 section, public test mnemonic, regtest)
derive_address_set(account=0, index=0, Regtest) = DerivedAddressSet {
    bip44_p2pkh: "mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV",
    bip49_p2sh_p2wpkh: "2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2",
    bip84_p2wpkh: "bcrt1q6rz28mcfaxtmd6v789l9rrlrusdprr9pz3cppk" }
recovery_is_repeatable(index 0, passphrase "class") = true
changing_index_changes_address(index 0 vs 1) = true
same path (m/44'/1'/0'/0/0), format P2PKH  = mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV
same path (m/44'/1'/0'/0/0), format P2WPKH = bcrt1q8gk5z3dy7zv9ywe7synlrk58elz4hrnegvpv6m
```

## Evidence references

- Implementation: [`src/labs/lab10_recovery.rs`](../src/labs/lab10_recovery.rs)
- Public test suite: [`tests/lab_10.rs`](../tests/lab_10.rs) — 4/4 passing, logged in
  [`grading/logs/lab_10.log`](../grading/logs/lab_10.log).
- Regtest prefixes recorded: BIP44 → `m`/`n` (P2PKH), BIP49 → `2` (P2SH-wrapped
  P2WPKH), BIP84 → `bcrt1q` (native P2WPKH) — matching Lab 01/05's family-to-prefix
  mapping.
- `bip44_p2pkh` above (`mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV`) is byte-for-byte the same
  address `derive_bip44_address` produced in Lab 09 from the identical path
  `m/44'/1'/0'/0/0`, mnemonic, and network — direct cross-lab evidence of repeatability.
- All derivations use only the published public test mnemonic.

## Explanation

`recovery_is_repeatable` derives the same address twice, independently, from identical
inputs (mnemonic, passphrase, path, format, network) and both runs match. This is not
an accident of implementation — it follows directly from BIP32/39 being pure functions
of their inputs: the seed is PBKDF2(mnemonic, passphrase) with no external state, and
every xpriv/xpub in the tree is HMAC-SHA512 of the parent key material and chain code
with no randomness anywhere in the path. There is nothing in the whole pipeline that
depends on wall-clock time, machine state, or prior calls — given the same four inputs,
the same private key, and therefore the same address, is produced every single time.
This determinism is the entire point of a "hierarchical **deterministic**" wallet: it
is what makes a wallet **recoverable** from nothing but the words and the passphrase,
without needing to separately back up any individual key.

`changing_index_changes_address` shows the complementary fact: changing only the final
child number (`.../0/0` → `.../0/1`) selects an entirely different, unrelated-looking
key — because, per Lab 08, the child index is hashed into the HMAC input at that step,
so even a one-unit change in the index produces a completely different 256-bit tweak,
and therefore a completely different key and address, with no partial correlation to
the previous one.

But recovering a wallet is not fully described by "same mnemonic ⇒ same keys." As
`derive_address_set`/`derive_address_for_path` show, the *same* private key at the
*same* path produces three visibly different addresses depending on which script
family it is locked into (P2PKH vs. P2SH-P2WPKH vs. P2WPKH) — `format_selection_changes_the_lock_target`
demonstrates exactly this for one path. So full recovery depends on **two** independent
conventions being reproduced correctly, not just the seed: (1) the **derivation path**
(which BIP44/49/84 purpose, coin type, account, and branch was used) and (2) the
**script family** each path's keys were locked into when funds were originally
received. A wallet that gets the mnemonic and passphrase right but assumes the wrong
purpose (say, always deriving BIP44/P2PKH) will generate correct-looking but entirely
different addresses from a wallet that actually received funds on BIP84/P2WPKH — the
keys underneath are identical, but the coins would appear "missing" until the matching
script-family convention is also restored.
