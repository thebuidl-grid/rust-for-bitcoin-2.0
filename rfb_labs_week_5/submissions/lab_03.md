# Lab 03 — P2SH 2-of-3 multisig

## Commands used

```bash
cargo test --test lab_03
cargo run -- 3
```

The runner builds a 2-of-3 redeemScript from three fixed test public keys, then prints
the redeemScript, the regtest P2SH address committing to it, the outer scriptPubKey and
the redeemScript size.

## Terminal output

```text
$ cargo test --test lab_03
running 4 tests
test builds_the_outer_p2sh_lock ... ok
test derives_the_committed_p2sh_address ... ok
test builds_a_two_of_three_redeem_script ... ok
test reports_both_validation_layers ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo run -- 3
== Lab 03: P2SH 2-of-3 multisig ==
  redeemScript  5221031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f21024d4b6cd1361032ca9bd2aeb9d900aa4d45d9ead80ac9423374c451a7254d07662102531fe6068134503d2723133227c867ac8fa6c83c537e9a44c3c5bdbdcb1fe33753ae
  address       2N99mC22Sz4sHLo6zSYkiCBmY47huZuMJbj
  scriptPubKey  a914ae79902ae33900b679c76ced8576362e4abb15e887
  redeemScript size 105 bytes
```

## Evidence references

Implementation in `src/labs/lab03_p2sh.rs`, tests in `tests/lab_03.rs`, runner in
`src/main.rs` under `lab03`.

The redeemScript hex reads straight off. `52` is OP_2. Then three `21` byte pushes of
compressed public keys. Then `53` for OP_3 and `ae` for OP_CHECKMULTISIG. That is
1 + (3 * 34) + 1 + 1 = 105 bytes, which matches the printed size and sits well under
the 520 byte limit Address::p2sh enforces.

The outer scriptPubKey `a914ae79902ae33900b679c76ced8576362e4abb15e887` carries none of
that. It is OP_HASH160, one 20-byte push, OP_EQUAL. Twenty-three bytes, whatever size
the inner script is. The address starts with `2`, which is Base58Check version byte
0xc4 for P2SH on the test networks.

## Explanation

P2SH validates in two layers. They run at different times and prove different things.

The outer layer is what the funding transaction records: OP_HASH160 <scriptHash>
OP_EQUAL. Those twenty-three bytes mention no keys, no threshold and no signers. At
spend time the input supplies the redeemScript as the final push of its ScriptSig. The
node hashes that push and compares it to the committed value. If they match, the outer
script succeeds. The outer layer does nothing else. It proves one thing, which is that
the script being revealed is the script the sender committed to.

The inner layer runs next. Under BIP16 the node deserializes the redeemScript it just
verified and executes it against the remaining ScriptSig items. OP_CHECKMULTISIG runs
here, pops the three public keys and the threshold, and requires two valid signatures
from two different keys, supplied in the same relative order as the keys in the script.
Reveal the correct redeemScript with only one signature, or two from the same key, or
two in the wrong order, and the hash check passes while the multisig check fails.

So matching the script hash is necessary and not close to sufficient. The redeemScript
becomes public the moment it is revealed, and everyone who helped build the address
knew it already. Anyone can assemble a ScriptSig whose final push hashes correctly.
Producing the two signatures is the part that needs private keys, and each signature is
bound to the specific transaction being signed.

The split also keeps the sender's job small. Whoever pays into
`2N99mC22Sz4sHLo6zSYkiCBmY47huZuMJbj` sees an ordinary Base58Check address and creates
an ordinary twenty-three byte output. They never find out it is multisig, how many
signers there are, or what the threshold is. BIP16 moved the cost of the policy onto
the receiver, and the size of the output stays fixed however large the redeemScript
gets.
