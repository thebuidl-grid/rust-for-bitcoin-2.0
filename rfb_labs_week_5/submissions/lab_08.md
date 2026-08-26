# Lab 08 — BIP32 extended keys

## Commands used

```bash
cargo test --test lab_08 -- --nocapture
cargo run --example labs_demo
```

## Terminal output

```text
$ cargo test --test lab_08 -- --nocapture
running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

```text
$ cargo run --example labs_demo   (Lab 08 section, public test mnemonic, empty passphrase, regtest)
master_xpriv(regtest) = Ok("tprv8ZgxMBicQKsPe5YMU9gHen4Ez3ApihUfykaqUorj9t6FDqy3nP6eoXiAo2ssvpAjoLroQxHqr3R5nE3a5dU3DHTjTgJDd7zrbniJr6nrCzd")
derive_extended_keys(m/84'/1'/0') =
  ExtendedKeyReport { derivation_path: "m/84'/1'/0'",
    xpriv: "tprv8fSjiqEQ8YG7Ro7gw2ScwcvweYuuWi1ZzGUtrPz918HvDtBzL5s2voFTrN4y3yUwj5cYD54pLhxk6NKCzHUjcka3zbKjbTEcsuAnkzbjhkL",
    xpub:  "tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M" }
derive_normal_child_xpub(parent.xpub, 7) =
  Ok("tpubDHPpQL4fzb9iBBVACHSYreu35GpBgZwyP4bFCwnyY9nTncRcbDQD9aDDo8fZFBmkAYLcoLNHws8N57iU15PpB72hmecwMcVhAuBWQEWs3Rr")
path_contains_hardened_step(m/44'/0'/0'/0/0) = Ok(true)
path_contains_hardened_step(m/0/1/2) = Ok(false)
```

## Evidence references

- Implementation: [`src/labs/lab08_bip32.rs`](../src/labs/lab08_bip32.rs)
- Public test suite: [`tests/lab_08.rs`](../tests/lab_08.rs) — 4/4 passing, logged in
  [`grading/logs/lab_08.log`](../grading/logs/lab_08.log).
- `xpriv`/`master_xpriv` outputs start with `tprv`, `xpub` outputs start with `tpub` —
  the correct BIP32 test-network version bytes for `Network::Regtest`.
- All values above derive solely from the published public BIP39 test mnemonic with an
  empty passphrase — the same class of disposable test material used throughout this
  assignment, never a real recovery seed. `derive_normal_child_xpub` demonstrates
  producing a *different* child xpub (`tpub...Rr`) from the parent xpub with no private
  key material involved at any point.

## Explanation

A BIP32 extended key is a public/private key plus a 32-byte **chain code**. The chain
code's job is to make derivation deterministic *and structured*, rather than just
deterministic: without it, deriving a "child" would just be re-hashing the parent key
with an index, which would make every child key a simple, guessable function of the
parent key and index alone. The chain code is combined with the (public or private) key
via HMAC-SHA512 at each derivation step, so it acts as extra entropy that is
carried down the tree, letting a whole tree of unrelated-looking keys be reproduced
deterministically from one seed plus a path — which is what lets a wallet regenerate
every account, chain, and address key it will ever need purely from the 12/24-word
mnemonic, with no need to back up each key individually.

An **xpub** carries the public key and chain code but not the private key. Because
`ckd_pub` (public-parent-to-public-child derivation) only needs the parent public key
and chain code to compute a child public key, an xpub lets a **watch-only** wallet,
block explorer, or payment-processing server generate every receiving address on that
branch — as `derive_normal_child_xpub` does above — without ever holding signing
authority over the funds those addresses receive.

That "public-to-public" derivation is only possible for **normal** (non-hardened)
children, and it stops working entirely at a **hardened** step, which is exactly what
`path_contains_hardened_step` is checking for. The reason is in the derivation formula
itself: a normal child's tweak is HMAC(chain_code, parent_public_key || index) — public
inputs only, so `ckd_pub` can compute it. A hardened child's tweak is instead HMAC
(chain_code, 0x00 || parent_private_key || index) — it deliberately mixes in the
*private* key, so only someone holding the parent xpriv can compute it. This is exactly
the guarantee hardened derivation exists to provide: if an attacker learns an xpub and
also learns one *non-hardened* child's private key, they can invert the HMAC tweak and
recover the parent xpriv directly. Hardened derivation breaks that attack, because
recovering the tweak would require already knowing the parent private key — which is
why BIP44/49/84 hardened everything from `purpose'` through `account'` and left only
`change`/`address_index` as normal, giving exactly the account-level xpub-safe,
address-level watch-only structure this lab and Lab 09 build on.
