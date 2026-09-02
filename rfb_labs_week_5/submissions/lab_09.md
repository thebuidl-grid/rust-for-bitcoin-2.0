# Lab 09 — BIP44 path decoding

## Commands used

```bash
cargo test --test lab_09
cargo run -- 9
```

The runner decodes `m/44'/0'/2'/1/5` level by level, explains it in English, replaces
only the final index, then derives the first three receive addresses and the first
change address on the regtest BIP44 branch.

## Terminal output

```text
$ cargo test --test lab_09
running 4 tests
test decodes_every_bip44_level ... ok
test changes_only_the_final_index ... ok
test explains_zero_based_account_and_chain ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

$ cargo run -- 9
== Lab 09: BIP44 path decoding ==
  path      m/44'/0'/2'/1/5
  decoded   purpose 44 coin_type 0 account 2 change 1 index 5
  meaning   purpose 44' fixes the BIP44 layout, coin type 0' fixes the chain, account 2' is the third account, chain 1 is the change branch, and index 5 is the sixth address on that branch
  index 5 -> 6: m/44'/0'/2'/1/6
  m/44'/1'/0'/0/0 -> mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV
  m/44'/1'/0'/0/1 -> mzpbWabUQm1w8ijuJnAof5eiSTep27deVH
  m/44'/1'/0'/0/2 -> mnTkxhNkgx7TsZrEdRcPti564yQTzynGJp
  m/44'/1'/0'/1/0 (change) -> mi8nhzZgGZQthq6DQHbru9crMDerUdTKva
```

## Evidence references

Implementation in `src/labs/lab09_bip44.rs`, tests in `tests/lab_09.rs`, runner in
`src/main.rs` under `lab09`.

The decode prints the five levels separately, and the English rendering turns the
zero-based numbers into the ordinals a person would use. Account 2 is the third
account and index 5 is the sixth address. Rewriting index 5 to 6 touches the last level
only and leaves `m/44'/0'/2'/1/` alone, which keeps the new address in the same account
and the same branch.

The four derived regtest addresses show the tree working. Indexes 0, 1 and 2 on chain 0
are three unrelated addresses in one account, and `m/44'/1'/0'/1/0` on chain 1 is a
fourth. All four start with `m` or `n`, Base58Check version byte 0x6f for P2PKH on the
test networks. derive_bip44_address decodes the path before doing key work, so a
malformed or non-BIP44 path fails early.

`examples/bip_vectors.rs` confirms the mainnet equivalent. `m/44'/0'/0'/0/0` from the
same mnemonic derives `1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA`.

## Explanation

`m/44'/0'/2'/1/5` has five levels below the master key, and each answers a different
question.

`44'` is the purpose and it fixes the layout, telling a wallet the remaining levels
follow BIP44 and the addresses are P2PKH. BIP49 and BIP84 reuse the same structure with
49 and 84 here, which is how one seed backs three address families without collision.
Lab 10 shows that.

`0'` is the coin type from SLIP44, where 0 is Bitcoin mainnet and 1 is every test chain.
That is why the mainnet examples use `44'/0'` and the regtest derivations above use
`44'/1'`. It keeps one seed's Bitcoin keys apart from its keys for other chains.

`2'` is the account, zero-based, so `2'` is the third one. Accounts are the intended
unit of separation for a user, personal and business funds in one wallet for example.
The level is hardened, so each account's xpub can be handed out without risk to the
others or to the master key.

`1` is the change level and takes two values by convention. Chain 0 is the external or
receive branch, the addresses handed to other people. Chain 1 is the internal or change
branch, where a wallet sends its own change. Keeping them apart keeps change out of the
address list shown to users, and it makes gap limit scanning workable since each branch
can be searched on its own.

`5` is the address index, also zero-based, so it picks the sixth address on that branch.
This level and the change level are normal, not hardened, which is what lets an account
xpub generate all of these addresses on a watch-only machine.

The apostrophes mark hardened derivation, meaning 2^31 is added to the child index and
the parent private key goes into the HMAC. Only the top three levels have them. Purpose,
coin type and account are hardened so a leaked child key cannot be walked back to the
master key. Change and address index stay normal so publishing an account xpub is still
useful. Zero-based indexing at both the account and address levels causes off-by-one
confusion in practice, which is why describe_bip44_path spells the ordinal out.
