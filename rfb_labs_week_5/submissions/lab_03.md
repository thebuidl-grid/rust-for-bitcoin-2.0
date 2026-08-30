# Lab 03 — P2SH 2-of-3 multisig and redeemScripts

## Commands used

```bash
RUSTUP_HOME=/home/dorine/.gemini/antigravity/scratch/rustup CARGO_HOME=/home/dorine/.gemini/antigravity/scratch/cargo cargo test --test lab_03
```

## Terminal output

```
running 4 tests
test derives_the_committed_p2sh_address ... ok
test builds_a_two_of_three_redeem_script ... ok
test builds_the_outer_p2sh_lock ... ok
test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Le code d'implémentation se trouve dans [lab03_p2sh.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab03_p2sh.rs). Les tests correspondants se trouvent dans [lab_03.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_03.rs).

## Explanation

Pour un script P2SH (Pay-to-Script-Hash), la transaction verrouille les fonds avec un hash externe (HASH160 du redeemScript) via le scriptPubKey `OP_HASH160 <script_hash> OP_EQUAL`. Lors de la dépense, le ScriptSig fournit d'abord les éléments de déverrouillage (signatures) suivis du redeemScript lui-même comme dernier élément. Le moteur de consensus effectue deux étapes de vérification :
1. Le "outer hash check" (vérification externe) : Il hache le redeemScript fourni et vérifie qu'il correspond exactement au hash dans le scriptPubKey.
2. Le "inner multisig check" (vérification interne multisig) : Si la première étape réussit, le redeemScript est désérialisé et exécuté comme un script actif en utilisant les signatures fournies dans le ScriptSig pour effectuer la vérification de multisignature (ici, 2-sur-3 via OP_CHECKMULTISIG).
