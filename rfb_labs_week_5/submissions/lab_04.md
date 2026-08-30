# Lab 04 — Native P2WPKH witness programs

## Commands used

```bash
RUSTUP_HOME=/home/dorine/.gemini/antigravity/scratch/rustup CARGO_HOME=/home/dorine/.gemini/antigravity/scratch/cargo cargo test --test lab_04
```

## Terminal output

```
running 4 tests
test leaves_scriptsig_empty_and_uses_witness ... ok
test builds_a_version_zero_witness_lock ... ok
test reports_a_twenty_byte_program ... ok
test derives_a_native_regtest_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Le code d'implémentation se trouve dans [lab04_p2wpkh.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab04_p2wpkh.rs). Les tests correspondants se trouvent dans [lab_04.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_04.rs).

## Explanation

Le format native P2WPKH (Segregated Witness BIP141) déplace les données de signature et de clé publique hors du ScriptSig traditionnel pour les placer dans un champ séparé appelé 'Witness' (témoin). Le ScriptSig est donc laissé vide pour deux raisons majeures :
1. Éliminer la malléabilité des transactions en empêchant les nœuds tiers de modifier les signatures à l'intérieur du ScriptSig (ce qui changerait l'ID de la transaction).
2. Réduire les frais de transaction, car les données du témoin (Witness) bénéficient d'un rabais de poids (weight discount) par rapport aux données du ScriptSig. De plus, les anciens nœuds (non-SegWit) voient le scriptPubKey comme un script 'anyone-can-spend' (que tout le monde peut dépenser) et n'exigent pas de ScriptSig, tandis que les nouveaux nœuds savent qu'ils doivent chercher les signatures dans le témoin.
