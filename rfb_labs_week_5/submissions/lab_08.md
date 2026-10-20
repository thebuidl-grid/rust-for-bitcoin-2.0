# Lab 08 — BIP32 extended keys and hardened derivation

## Commands used

```bash
RUSTUP_HOME=/home/dorine/.gemini/antigravity/scratch/rustup CARGO_HOME=/home/dorine/.gemini/antigravity/scratch/cargo cargo test --test lab_08
```

## Terminal output

```
running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

## Evidence references

Le code d'implémentation se trouve dans [lab08_bip32.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab08_bip32.rs). Les tests correspondants se trouvent dans [lab_08.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_08.rs).

## Explanation

- xpriv (extended private key) : Contient une clé privée et un chain code. Elle permet de dériver d'autres clés privées descendantes et leurs clés publiques correspondantes.
- xpub (extended public key) : Contient une clé publique et un chain code. Elle permet de dériver uniquement des clés publiques descendantes, sans jamais révéler les clés privées associées.
- Chain code (code de chaîne) : 32 octets de données aléatoires utilisés comme sel entropique pour garantir que les dérivations successives restent cryptographiquement indépendantes les unes des autres.
- Dérivation normale (Normal derivation) : Dérive un enfant à partir de la clé publique parente et du chain code. Un compromis de sécurité majeur existe : si un attaquant obtient une clé privée descendante et la clé xpub parente, il peut reconstruire la clé xpriv parente.
- Dérivation renforcée (Hardened derivation) : Dérive un enfant en utilisant la clé privée parente (et non la clé publique). Cela empêche la dérivation d'enfants à partir de la clé publique parente uniquement (impossible d'utiliser xpub), éliminant ainsi le risque de fuite de clé parente en cas de compromission d'une clé enfant.
