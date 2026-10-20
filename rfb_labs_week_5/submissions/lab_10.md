# Lab 10 — Deterministic recovery across BIP44, BIP49, and BIP84

## Commands used

```bash
RUSTUP_HOME=/home/dorine/.gemini/antigravity/scratch/rustup CARGO_HOME=/home/dorine/.gemini/antigravity/scratch/cargo cargo test --test lab_10
```

## Terminal output

```
running 4 tests
test format_selection_changes_the_lock_target ... ok
test identical_recovery_inputs_repeat ... ok
test derives_three_regtest_address_families ... ok
test changing_only_the_index_changes_the_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```

## Evidence references

Le code d'implémentation se trouve dans [lab10_recovery.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab10_recovery.rs). Les tests correspondants se trouvent dans [lab_10.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_10.rs).

## Explanation

La récupération déterministe d'un portefeuille repose sur des entrées standardisées et des conventions de dérivation strictes. En fournissant la même phrase mnémonique et phrase de passe BIP39, on obtient de manière déterministe la même graine binaire de 512 bits. Ensuite, en appliquant les chemins de dérivation BIP32 standardisés (BIP44/49/84), le portefeuille peut régénérer exactement la même suite de clés privées et publiques. Comme le processus est entièrement déterministe, il suffit de parcourir séquentiellement les index d'adresses (par exemple, de 0 à 20, selon le principe du gap limit) pour retrouver l'historique complet des transactions et reconstituer le solde du portefeuille sans avoir besoin d'aucune sauvegarde externe des clés générées.
