# Lab 01 — Address and network identification

## Commands used

```bash
RUSTUP_HOME=/home/dorine/.gemini/antigravity/scratch/rustup CARGO_HOME=/home/dorine/.gemini/antigravity/scratch/cargo cargo test --test lab_01
```

## Terminal output

```
running 4 tests
test identifies_human_readable_prefixes ... ok
test maps_regtest_prefixes ... ok
test rejects_an_address_for_the_wrong_network ... ok
test inspects_a_network_checked_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Le code source de l'implémentation est disponible dans [lab01_addresses.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab01_addresses.rs). Les tests validant l'adresse se trouvent dans [lab_01.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_01.rs).

## Explanation

L'inspection du préfixe seul (comme '1', '3', 'bc1') n'est pas une validation d'adresse complète car elle ne vérifie pas la somme de contrôle (checksum) de l'adresse ni sa longueur ou sa validité structurelle. De plus, elle ne garantit pas que les octets décodés représentent un scriptPubkey valide sur le réseau ciblé (par exemple, des caractères invalides pourraient être présents ou le hash décodé pourrait ne pas avoir la bonne taille).
