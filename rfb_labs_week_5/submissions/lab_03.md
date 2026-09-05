# Lab 03 — P2SH 2-of-3 multisig

# Lab 03 — P2SH 2-of-3 multisig

## Commands used
```
cargo fmt
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --test lab_03
```
## Terminal output
```
running 4 tests
test builds_the_outer_p2sh_lock ... ok
test derives_the_committed_p2sh_address ... ok
test builds_a_two_of_three_redeem_script ... ok
test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
## Evidence references

The public Lab 03 test output above is the primary execution evidence for this lab.

The tests verify that:

- a canonical 2-of-3 multisig redeemScript is constructed
- a P2SH address is derived from that redeemScript
- the outer P2SH scriptPubKey is constructed correctly
- the resulting report connects the redeemScript, P2SH address, and outer scriptPubKey


## Explanation

This task combines two ideas: **multisignature authorization** and **Pay to Script Hash (P2SH)**.

The first part is the 2-of-3 multisig rule. The redeemScript constructed by
`build_2_of_3_redeem_script` is:

2 <pubkey1> <pubkey2> <pubkey3> 3 OP_CHECKMULTISIG

The `2` at the beginning means that two signatures are required. The following
three values are the public keys that are allowed to participate in the spend.
The `3` tells the script that there are three public keys in total, and
`OP_CHECKMULTISIG` performs the signature checks.

So this does not mean that all three people must sign. It means that any two
valid signatures from the three corresponding private keys can satisfy the
spending condition. For example, if three people each control one private key,
two of them can cooperate to spend the output without requiring the third person.

The redeemScript is the actual spending rule, but P2SH changes where that rule
is committed. Instead of putting the entire multisig script directly into the
output's scriptPubKey, P2SH hashes the redeemScript using HASH160.

The outer P2SH scriptPubKey is:

OP_HASH160 <HASH160(redeemScript)> OP_EQUAL

`build_p2sh_script_pubkey` constructs this outer locking script. The output
therefore commits to the hash of the redeemScript rather than exposing the full
multisig rule directly in the output.

`derive_p2sh_address` performs the same commitment at the address level. It takes
the redeemScript, creates its P2SH address, and encodes that commitment for the
requested network. Because the tests use Regtest, the resulting P2SH address
starts with `2`.

There are therefore two different scripts involved:

1. The **scriptPubKey** is the outer P2SH locking script. It commits to a hash of
   the redeemScript.
2. The **redeemScript** is the inner script that contains the actual 2-of-3
   multisig spending rule.

When the output is later spent, the spender provides the redeemScript along with
the required signatures. Bitcoin first checks that the supplied redeemScript
hashes to the same value committed in the P2SH scriptPubKey. This proves that the
spender is using the exact spending rule that the output committed to.

After that commitment check succeeds, Bitcoin evaluates the redeemScript itself.
`OP_CHECKMULTISIG` then checks whether enough valid signatures have been provided
for the three public keys. At least two valid signatures are required.

This is why `inspect_p2sh_multisig` is useful: its report contains both the
redeemScript and the outer P2SH information, allowing us to see the two validation
layers separately.

The key idea is that **P2SH answers "which spending script was committed to?",
while the redeemScript answers "what conditions must be satisfied to spend?"**.
In this task, those conditions are that two of the three authorized keys provide
valid signatures.