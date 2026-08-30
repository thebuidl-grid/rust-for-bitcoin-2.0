# Lab 02 — Legacy P2PKH

## Commands used

```bash
RUSTUP_HOME=/home/dorine/.gemini/antigravity/scratch/rustup CARGO_HOME=/home/dorine/.gemini/antigravity/scratch/cargo cargo test --test lab_02
```

## Terminal output

```
running 4 tests
test commits_to_hash160_of_the_public_key ... ok
test puts_unlocking_data_in_scriptsig ... ok
test builds_the_standard_p2pkh_lock ... ok
test derives_the_expected_p2pkh_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Le code d'implémentation se trouve dans [lab02_p2pkh.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab02_p2pkh.rs). Les tests unitaires et d'intégration correspondants se trouvent dans [lab_02.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_02.rs).

## Explanation

P2PKH (Pay-to-Public-Key-Hash) verrouille les fonds à un hash de clé publique (HASH160). Le scriptPubKey contient la formule standard `OP_DUP OP_HASH160 <pubkey_hash> OP_EQUALVERIFY OP_CHECKSIG`. Pour déverrouiller et dépenser ces fonds, le ScriptSig du dépensier doit fournir une signature valide et la clé publique correspondante. Le script concaténé duplique la clé publique, la hache, la compare au hash verrouillé (OP_EQUALVERIFY), puis utilise la signature fournie pour vérifier que le dépensier possède la clé privée associée à cette clé publique (OP_CHECKSIG).
