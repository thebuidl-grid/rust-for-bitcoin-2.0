# Lab 08 — BIP32 extended keys

## Commands used

```bash
cargo test --test lab_08
cargo run -- 8
```

The runner creates the regtest master key from the public class mnemonic, derives the
`m/84'/1'/0'` account and the `m/84'/1'/0'/0` branch, derives child index 7 from the
branch xpub alone, and classifies three paths as hardened or normal.

## Terminal output

```text
$ cargo test --test lab_08
running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

$ cargo run -- 8
== Lab 08: BIP32 extended keys ==
  master xpriv  tprv8Zgx...rCzd (masked)
  path          m/84'/1'/0'
  account xpriv tprv8fSj...jhkL (masked)
  account xpub  tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M
  branch xpub   tpubDFd87GgwwqSRLStzXaBq4mJvFm8e9quGD5L9E5XMhtJnJUgFGErWJFwgBr9RLyXGdzDhfAChbKF6p2RaZsArrJAgAhTWNWFDWyDkshPRodD
  child 7 xpub  tpubDHPpQL4fzb9iBBVACHSYreu35GpBgZwyP4bFCwnyY9nTncRcbDQD9aDDo8fZFBmkAYLcoLNHws8N57iU15PpB72hmecwMcVhAuBWQEWs3Rr
  m/84'/1'/0'/0/0 hardened step: true
  m/0/1/2 hardened step: false
  not/a/path: rejected: invalid derivation path: invalid child number format
```

## Evidence references

Implementation in `src/labs/lab08_bip32.rs`, tests in `tests/lab_08.rs`, runner in
`src/main.rs` under `lab08`.

Extended private keys are masked to their first eight and last four characters. They
come from the published test mnemonic so they are not sensitive, but the version prefix
is the only part the evidence needs. The xpubs are printed in full since they carry no
spending authority.

The prefixes confirm the network selection. Regtest shares `tprv` and `tpub`
serialization with the other test chains, where mainnet would show `xprv` and `xpub`.
The child xpub at index 7 differs from its parent and was produced without any private
key. `not/a/path` is rejected before derivation starts, instead of being treated as the
master path.

## Explanation

A BIP32 extended key is a keypair plus a 32-byte chain code, and the chain code is what
makes the tree work. Child derivation feeds the parent key and the chain code into
HMAC-SHA512. The left half of the output is a tweak added to the parent key and the
right half becomes the child's chain code. Without it the derivation would not be
reproducible, and an attacker holding one child private key and the parent public key
still could not walk the tree. So an xpub is not just a public key. It carries enough
state to generate unlimited descendant public keys, and it should be treated as private
data even though it cannot sign.

That property is what makes watch-only operation possible. A point of sale system, an
exchange deposit service or an accounting tool can hold `m/84'/1'/0'/0` as an xpub and
generate a fresh receive address per customer indefinitely, while the private keys stay
offline. It can recognise incoming payments, compute balances and build unsigned
transactions. It cannot spend, and a full compromise of that server leaks address
history and future addresses but no funds. The output shows this: child index 7 came
from the branch xpub with no private key present.

Hardened children cannot come from a parent xpub, because hardened derivation feeds the
parent private key into the HMAC in place of the parent public key. Child indexes from
2^31 upward are reserved for it. from_normal_idx enforces the boundary and errors
instead of wrapping, which my implementation surfaces as a Derivation error.

The restriction exists because of a real attack. In normal derivation the child private
key is the parent private key plus a tweak that anyone with the parent xpub can compute.
If one normal child private key leaks and the attacker has the parent xpub, they
subtract the tweak, recover the parent private key, and derive every key in the branch.
A single leaked address key compromises the whole account. Hardened derivation breaks
that arithmetic link, so a leaked hardened child says nothing about its parent or its
siblings.

This is why BIP44 hardens the top three levels and leaves the bottom two normal.
Purpose, coin type and account are hardened, so publishing an account xpub cannot
endanger the master key or a sibling account. Change and address index stay normal so
that account xpub is still useful for watch-only address generation. The output
confirms the split: `m/84'/1'/0'/0/0` reports a hardened step and `m/0/1/2` does not.
