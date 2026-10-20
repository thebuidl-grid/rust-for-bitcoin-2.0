# Lab 07 — BIP39 mnemonics, seeds, and passphrases

## Commands used

```bash
RUSTUP_HOME=/home/dorine/.gemini/antigravity/scratch/rustup CARGO_HOME=/home/dorine/.gemini/antigravity/scratch/cargo cargo test --test lab_07
```

## Terminal output

```
running 4 tests
test rejects_an_invalid_checksum ... ok
test validates_entropy_and_checksum_structure ... ok
test matches_the_published_bip39_seed_vector ... ok
test passphrase_selects_a_different_wallet ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

## Evidence references

Le code d'implémentation se trouve dans [lab07_bip39.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/src/labs/lab07_bip39.rs). Les tests correspondants se trouvent dans [lab_07.rs](file:///home/dorine/.gemini/antigravity/scratch/rust-for-bitcoin-2.0/rfb_labs_week_5/tests/lab_07.rs).

## Explanation

- L'entropie (Entropy) est une suite d'octets aléatoires générée de manière sécurisée (par exemple 128 ou 256 bits) qui sert de source initiale de hasard pour le portefeuille.
- La somme de contrôle (Checksum) est calculée en hachant l'entropie pour détecter les erreurs de saisie lors de la récupération de la phrase mnémonique.
- La phrase mnémonique (Mnemonic) est une répresentation lisible par l'homme de l'entropie combinée à sa somme de contrôle, exprimée sous forme d'une suite de mots (généralement 12 à 24 mots) tirés d'un dictionnaire fixe (BIP39).
- La phrase de passe (Passphrase) est un mot de passe optionnel ajouté par l'utilisateur pour dériver une clé graine totalement différente à partir du même mnémonique (fournissant un second facteur de sécurité ou 'plausible deniability').
- La clé graine (Seed) est une suite binaire de 512 bits générée par dérivation PBKDF2 à partir du mnémonique et de la phrase de passe. C'est à partir de cette graine que l'arbre de clés HD (BIP32) est généré.
