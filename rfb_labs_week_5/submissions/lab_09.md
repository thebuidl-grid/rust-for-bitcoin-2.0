# Lab 09 — BIP44 path decoding and address derivation

## Commands used

```bash
RUSTUP_HOME=/home/dorine/.gemini/antigravity/scratch/rustup CARGO_HOME=/home/dorine/.gemini/antigravity/scratch/cargo cargo test --test lab_09
```

## Terminal output

```
running 4 tests
test decodes_every_bip44_level ... ok
test changes_only_the_final_index ... ok
test explains_zero_based_account_and_chain ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

## Evidence references

Le code d'implémentation se trouve dans [lab09_bip44.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab09_bip44.rs). Les tests correspondants se trouvent dans [lab_09.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_09.rs).

## Explanation

Le chemin de dérivation standard BIP44 s'écrit sous la forme `m / purpose' / coin_type' / account' / change / address_index` :
- Purpose (Objectif, e.g., 44') : Indique la spécification de dérivation utilisée (BIP44 pour Legacy, BIP49 pour Wrapped SegWit, BIP84 pour Native SegWit).
- Coin Type (Type de monnaie, e.g., 0' pour Bitcoin Mainnet, 1' pour Testnet) : Identifie le réseau ou la cryptomonnaie cible pour éviter de réutiliser des clés d'un réseau sur un autre.
- Account (Compte, e.g., 0', 1') : Permet de segmenter le portefeuille en plusieurs comptes distincts pour l'organisation ou la comptabilité des fonds.
- Change (Chaîne de réception ou de rendu, e.g., 0 pour la réception externe, 1 pour les adresses de rendu de monnaie interne).
- Address Index (Index de l'adresse, e.g., 0, 1, 2...) : L'identifiant séquentiel de l'adresse générée sur la chaîne spécifiée.
