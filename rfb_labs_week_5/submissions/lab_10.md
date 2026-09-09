# Lab 10 — Deterministic recovery across address families

## Commands used

```bash
cargo test --test lab_10
bash grader/grade.sh
```


## Terminal output

running 4 tests
test identical_recovery_inputs_repeat ... ok
test changing_only_the_index_changes_the_address ... ok
test format_selection_changes_the_lock_target ... ok
test derives_three_regtest_address_families ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

## Evidence references

All four public tests in `tests/lab_10.rs` pass, using only the published
public BIP39 test mnemonic:
- Deriving one address on each of BIP44, BIP49, and BIP84 at index 0, and
  confirming each carries the correct regtest prefix for its script family
  (`derives_three_regtest_address_families`)
- Re-deriving the same mnemonic, passphrase, path, format, and network
  twice and confirming the result is identical
  (`identical_recovery_inputs_repeat`)
- Deriving at two paths differing only in the final index and confirming
  the resulting addresses differ (`changing_only_the_index_changes_the_address`)
- Deriving the same path under two different script formats and confirming
  the resulting addresses differ (`format_selection_changes_the_lock_target`)

## Explanation

Identical recovery inputs reproduce the same address because every step in
this pipeline is a deterministic function: the same mnemonic and
passphrase always produce the same 512-bit seed (BIP39, Lab 07), the same
seed always produces the same master key and chain code (BIP32, Lab 08),
and walking the same derivation path from that master always visits the
same sequence of child keys. There is no randomness anywhere after the
initial mnemonic is generated — recovery is really just re-running this
same deterministic pipeline on a different machine, which is the entire
point of an HD wallet: back up twelve words once, and every key the wallet
ever used or will use can be reconstructed exactly.

But recovering the same *keys* is not the same as recovering the same
*wallet*, because the mnemonic and passphrase alone don't specify which
derivation paths or script types were actually used to receive funds. This
lab shows that one seed can produce three completely different, valid
addresses at the "same" index — a BIP44 P2PKH address, a BIP49
P2SH-wrapped-P2WPKH address, and a BIP84 native P2WPKH address — because
each standard defines its own purpose-level branch (`44'`, `49'`, `84'`) and
its own way of turning a derived public key into an address. If a wallet
originally received funds at a BIP84 address but a recovery attempt only
scans BIP44 paths, it will find nothing and appear empty, even though the
mnemonic and passphrase are entirely correct. Successfully restoring a
wallet therefore depends on knowing — or correctly guessing, or having it
recorded — not just the recovery words, but the specific derivation-path
and script-type conventions the original wallet software used.

