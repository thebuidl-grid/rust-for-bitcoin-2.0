# Lab 06 — Weight, virtual size, and fees

## Commands used

TODO: List the Rust commands you ran.

## Terminal output

TODO: Record weight, vsize, both fees, and savings.

## Evidence references

TODO: Link screenshots or describe attached evidence.

## Explanation

Transaction weight measures consumption of Bitcoin's limited block capacity. Base
transaction bytes count as four weight units each, while witness bytes count as one
weight unit each. If `stripped_size` excludes witness data and `total_size` includes
it, the BIP141 formula is
`weight = stripped_size * 3 + total_size`. This is equivalent to counting stripped
bytes four times and witness-only bytes once.

Virtual size is `ceil(weight / 4)`, and a fee at a quoted sat/vB rate is
`virtual_size * feerate`. In the lab comparison, a 226-vB legacy transaction costs
11,300 sats at 50 sat/vB, while a 141-vB P2WPKH transaction costs 7,050 sats, saving
4,250 sats.

This is not a flat discount applied to the whole SegWit transaction, and witness
data is neither removed nor ignored. Nodes still receive and validate the witness,
including its signatures. Only witness bytes receive the lower weight factor;
version fields, inputs, outputs, amounts, and other stripped transaction data retain
the full factor. Weight is principally block-space accounting rather than an exact
measurement of validation CPU work. It reflects scarce network resources such as
block capacity, relay bandwidth, storage burden, and validation workload while
making witness-based spends more fee-efficient.
