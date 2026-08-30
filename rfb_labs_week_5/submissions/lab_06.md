# Lab 06 — Transaction weight, virtual size, and fees

## Commands used

```bash
RUSTUP_HOME=/home/dorine/.gemini/antigravity/scratch/rustup CARGO_HOME=/home/dorine/.gemini/antigravity/scratch/cargo cargo test --test lab_06
```

## Terminal output

```
running 4 tests
test calculates_bip141_weight ... ok
test calculates_fee_from_feerate ... ok
test reproduces_the_class_fee_comparison ... ok
test rounds_weight_up_to_virtual_bytes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Le code d'implémentation se trouve dans [lab06_weight_fees.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab06_weight_fees.rs). Les tests correspondants se trouvent dans [lab_06.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_06.rs).

## Explanation

La comptabilisation du poids SegWit (BIP141) mesure la taille des transactions en 'unités de poids' (Weight Units ou WU) plutôt qu'en octets physiques. Une transaction se compose de deux parties : les données de base (taille dépouillée ou stripped size, excluant les témoins) et les données de témoin (witness). Chaque octet des données de base compte pour 4 WU, tandis que chaque octet des données de témoin compte pour seulement 1 WU. Le poids total est ainsi calculé par la formule `poids = (taille_dépouillée * 3) + taille_totale`. Le virtual size (vByte) est égal au poids divisé par 4 (arrondi à l'entier supérieur). Cela reflète le coût différentiel pour le stockage à long terme sur l'état du réseau (UTXO set), qui est plus coûteux que les signatures à usage unique (placées dans le témoin et élaguées par la suite).
