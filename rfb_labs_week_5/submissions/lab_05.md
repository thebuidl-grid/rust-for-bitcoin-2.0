# Lab 05 — Legacy, wrapped SegWit, native SegWit, and Taproot compatibility

## Commands used

```bash
RUSTUP_HOME=/home/dorine/.gemini/antigravity/scratch/rustup CARGO_HOME=/home/dorine/.gemini/antigravity/scratch/cargo cargo test --test lab_05
```

## Terminal output

```
running 4 tests
test builds_the_four_format_map ... ok
test older_p2sh_wallet_accepts_wrapped_but_not_native ... ok
test names_the_required_human_encoding ... ok
test selects_the_most_modern_supported_format ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Le code d'implémentation se trouve dans [lab05_compatibility.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab05_compatibility.rs). Les tests correspondants se trouvent dans [lab_05.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_05.rs).

## Explanation

Une adresse commençant par '3' est une adresse P2SH (Pay-to-Script-Hash), standardisée depuis 2012 (BIP16) et encodée en Base58Check. Un portefeuille de l'ère P2SH comprend et sait décoder le format Base58Check pour les adresses '3'. En revanche, les adresses native SegWit (P2WPKH) commençant par 'bc1q' utilisent un tout nouvel encodage appelé Bech32 (BIP173). Les portefeuilles anciens ne disposent pas de l'algorithme nécessaire pour décoder et valider le Bech32. Ainsi, ils rejettent 'bc1q...' comme une chaîne de caractères invalide ou inconnue, alors qu'ils acceptent les adresses '3' (y compris celles enveloppant du SegWit) car elles réutilisent le format P2SH existant.
