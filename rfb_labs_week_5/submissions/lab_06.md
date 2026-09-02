# Lab 06 — Weight, virtual size, and fees

## Commands used

```bash
cargo test --test lab_06
cargo run -- 6
```

The runner computes BIP141 weight for a one-input, two-output transaction in legacy and
native SegWit form, converts each to virtual bytes, and reproduces the class fee
comparison at 50 sat/vB. The last line shows the rejection when the stripped size
exceeds the total size.

## Terminal output

```text
$ cargo test --test lab_06
running 4 tests
test calculates_bip141_weight ... ok
test calculates_fee_from_feerate ... ok
test reproduces_the_class_fee_comparison ... ok
test rounds_weight_up_to_virtual_bytes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo run -- 6
== Lab 06: Weight, virtual size, and fees ==
  legacy 226 stripped / 226 total -> 904 wu -> 226 vB
  segwit 113 stripped / 222 total -> 561 wu -> 141 vB
  at 50 sat/vB: legacy 226 vB = 11300 sats, segwit 141 vB = 7050 sats, saving 4250 sats
  fee_sats(141, 50) = 7050
  transaction_weight(201, 200) = rejected: invalid transaction size: stripped size 201 exceeds total size 200
```

## Evidence references

Implementation in `src/labs/lab06_weight_fees.rs`, tests in `tests/lab_06.rs`, runner in
`src/main.rs` under `lab06`.

Both size pairs are the standard one-input, two-output templates. The legacy
transaction has no witness, so stripped and total are both 226 bytes and weight is
226 * 3 + 226 = 904, giving 226 vB. The SegWit transaction keeps 113 non-witness bytes
(4 version, 1 input count, 41 input with an empty ScriptSig, 1 output count, 62 for two
P2WPKH outputs, 4 locktime) and adds 109 witness bytes including marker and flag, so
total is 222 and weight is 113 * 3 + 222 = 561, giving ceil(561 / 4) = 141 vB.

At 50 sat/vB that gives 11,300 sats against 7,050 sats and a saving of 4,250, matching
the class figures. The public test pins the rounding boundary as well: 564 weight units
give 141 vB and 565 give 142.

## Explanation

Weight exists because segregated witness had to make witness data cheaper without
changing the one megabyte limit older nodes enforce. Both had to hold at once, and that
rules out the two obvious designs.

Witness data cannot be dropped from the calculation. Free witness bytes cost an attacker
nothing, so a block could be padded with huge witnesses until bandwidth, storage and
validation assumptions broke across the network. Every byte a node downloads and
verifies has to be paid for. The witness discount is a discount and never an exemption.

One flat discount across the whole transaction does not work either, because the two
kinds of data cost different amounts over time. Non-witness bytes define the UTXO set.
Outputs created there have to be tracked by every node indefinitely. Witness bytes are
signatures and public keys proving authorization once. A node doing initial block
download can skip verifying old signatures below its assumevalid point, and pruning
nodes discard them. One rate for both would misprice at least one of them.

BIP141 prices them separately and reports a single number. Weight is
stripped_size * 3 + total_size, which is the same as stripped_size * 4 + witness_bytes.
Non-witness bytes cost four weight units, witness bytes cost one, and the four
megaweight cap is exactly the old one megabyte limit for a transaction with no witness.
Virtual size is ceil(weight / 4), which puts weight back into units comparable with
pre-SegWit byte sizes so existing fee estimation keeps working. The rounding goes up so
nothing is billed for less capacity than it occupies.

The 226 vB against 141 vB comparison comes out of this and not out of the transactions
being different sizes. On the wire they are 226 and 222 bytes, almost the same. The
saving comes from moving the signature and public key across the witness boundary,
where their price drops from four to one.
