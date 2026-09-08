# Lab 09 — BIP44 path decoding

## Commands used

```bash
cargo test --test lab_09
cargo fmt --check
bash grader/grade.sh
```

## Terminal output

```text
running 4 tests
test changes_only_the_final_index ... ok
test decodes_every_bip44_level ... ok
test explains_zero_based_account_and_chain ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

## Evidence references

Automated unit tests in `tests/lab_09.rs` verifying 5-level BIP44 path parsing (`m/44'/0'/2'/1/5`), zero-based account and chain descriptions, index replacement (`with_address_index`), and P2PKH address derivation from public test recovery data (`m/n...` on regtest).

## Explanation

BIP44 establishes a standard 5-level path hierarchy `m / purpose' / coin_type' / account' / change / index`:
- **`purpose'`** (hardened, `44'`): Identifies the BIP specification governing the wallet structure (44 for legacy P2PKH).
- **`coin_type'`** (hardened, `0'` for Bitcoin Mainnet, `1'` for Testnet/Regtest): Separates different blockchain assets derived from the same master seed.
- **`account'`** (hardened, 0-indexed): Splits the wallet into independent accounts (e.g. `2'` represents the 3rd account). Hardened derivation prevents keys in one account from compromising other accounts.
- **`change`** (unhardened, `0` or `1`): Distinguishes the public receiving branch (`0`) for invoice generation from the private internal change branch (`1`) for transaction change outputs.
- **`index`** (unhardened, 0-indexed): Sequential address index generated on the specified branch (e.g. `5` represents the 6th address).
