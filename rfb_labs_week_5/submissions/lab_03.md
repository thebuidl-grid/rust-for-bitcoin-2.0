# Lab 03 — P2SH 2-of-3 multisig

## Commands used

TODO: List the Rust commands you ran.

## Terminal output

TODO: Record the redeemScript, P2SH address, and outer scriptPubKey.

## Evidence references

TODO: Link screenshots or describe attached evidence.

## Explanation

The 2-of-3 redeemScript contains the actual spending rule:
`2 <pubkey1> <pubkey2> <pubkey3> 3 OP_CHECKMULTISIG`. It requires valid signatures
for at least two of the three listed public keys. The order and serialization of
these keys matter because changing any byte produces a different script hash.

The outer P2SH scriptPubKey is
`OP_HASH160 <HASH160(redeemScript)> OP_EQUAL`. It does not contain or directly
execute the multisig policy when the output is created. When spending, the spender
reveals the redeemScript and supplies the required signatures. Bitcoin first hashes
the revealed script and checks that it matches the outer commitment. It then
executes that redeemScript and enforces the inner 2-of-3 signature rule. Matching
the script hash proves that the intended rule was revealed, but it does not by
itself satisfy that rule; two valid signatures are still required.
