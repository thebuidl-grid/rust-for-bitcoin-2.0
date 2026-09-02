# Lab 10 — Deterministic recovery

## Commands used

```bash
cargo test --test lab_10
cargo run -- 10
cargo run --example bip_vectors
```

The runner derives index 0 on the BIP44, BIP49 and BIP84 receive branches from one
mnemonic, repeats the derivation, changes only the final index, then derives the same
path under a different script family and under a different passphrase. The example
checks four published BIP vectors.

## Terminal output

```text
$ cargo test --test lab_10
running 4 tests
test changing_only_the_index_changes_the_address ... ok
test identical_recovery_inputs_repeat ... ok
test format_selection_changes_the_lock_target ... ok
test derives_three_regtest_address_families ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

$ cargo run -- 10
== Lab 10: Deterministic recovery ==
  index 0
    BIP44 m/44'/1'/0'/0/0 P2PKH        mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV
    BIP49 m/49'/1'/0'/0/0 P2SH-P2WPKH  2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2
    BIP84 m/84'/1'/0'/0/0 P2WPKH       bcrt1q6rz28mcfaxtmd6v789l9rrlrusdprr9pz3cppk
  same inputs reproduce the set: true
  index 1
    BIP44 mzpbWabUQm1w8ijuJnAof5eiSTep27deVH
    BIP49 2N55m54k8vr95ggehfUcNkdbUuQvaqG2GxK
    BIP84 bcrt1qd7spv5q28348xl4myc8zmh983w5jx32cs707jh
  repeatable at m/84'/1'/0'/0/0 with passphrase: true
  index change alters the address: true
  same key m/44'/1'/0'/0/0 as P2WPKH instead: bcrt1q8gk5z3dy7zv9ywe7synlrk58elz4hrnegvpv6m
  same path with the "class" passphrase: bcrt1qzdmet5f9x6s4lpmszpf5uzv0qa4hd3faa58l45
```

## Evidence references

Implementation in `src/labs/lab10_recovery.rs`, tests in `tests/lab_10.rs`, runner in
`src/main.rs` under `lab10`, vector check in `examples/bip_vectors.rs`.

The three prefixes mark the three script families. `m` or `n` is Base58Check P2PKH on
the test networks, `2` is Base58Check P2SH holding a wrapped witness program, and
`bcrt1q` is a bech32 version 0 witness program on regtest.

Two independent checks show the derivations are correct and not just self-consistent.
`2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2` at `m/49'/1'/0'/0/0` is the published BIP49 test
vector, reproduced exactly. And `cargo run --example bip_vectors` matches four
published values from the same mnemonic:

```text
  MATCH    BIP39 seed (passphrase TREZOR)
           derived  c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a6987599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04
  MATCH    BIP44 m/44'/0'/0'/0/0
           derived  1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA
  MATCH    BIP49 m/49'/1'/0'/0/0
           derived  2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2
  MATCH    BIP84 m/84'/0'/0'/0/0
           derived  bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu
  MATCH    BIP84 m/84'/0'/0'/0/1
           derived  bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g
```

The last two lines of the lab output isolate the two variables. Deriving
`m/44'/1'/0'/0/0` as P2WPKH instead of P2PKH gives a different address from the same
key. Deriving `m/84'/1'/0'/0/0` with the `class` passphrase gives a different address
from a different key on the same path.

## Explanation

Recovery is deterministic because every step from the words to the address is a pure
function with no randomness and no stored state.

The mnemonic and passphrase go through PBKDF2-HMAC-SHA512 to a fixed 512-bit seed. The
seed goes through HMAC-SHA512 with the fixed key "Bitcoin seed" to a master key and
chain code. Each path level applies HMAC-SHA512 again. The final public key is hashed
and encoded for the chosen script family and network. No step consults a random number
generator, a clock, a device identifier or anything the wallet saved earlier. Same
inputs, same outputs, on any machine and in any implementation. The
`same inputs reproduce the set: true` line tests that directly, and the BIP49 vector
match shows it holds across implementations too.

Every input matters, and the output changes each one in turn. Moving the final index
from 0 to 1 gives three entirely different addresses. Changing the script family turns
one derived key into a different address. Changing the passphrase gives a different key
on the same path. Nothing partially matches.

This is why a mnemonic on its own is an incomplete backup. The words fix the keys, but
the addresses also depend on conventions kept outside the words: purpose, coin type,
account number, branch, index, script family and network. A wallet restoring the class
mnemonic under BIP44 finds `mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV` and its history. The
same mnemonic restored under BIP84 finds
`bcrt1q6rz28mcfaxtmd6v789l9rrlrusdprr9pz3cppk`, sees nothing on the BIP44 branch and
reports an empty wallet. The funds are still there. The software looked in the wrong
branch.

Three things follow for practice. Record the derivation scheme next to the words, since
`84'` and P2WPKH are not secrets. Record whether a passphrase was used, because a wallet
cannot detect a wrong one and will show an empty balance instead of an error. And prefer
wallets that scan several standard branches on restore.
