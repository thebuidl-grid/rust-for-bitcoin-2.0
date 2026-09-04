# Lab 06 — Weight, virtual size, and fees

## Commands used

`cargo test --test lab_06`

## Terminal output

All 4 tests passed. The sample calculation gives weight 500, virtual size 141 for weight 564, legacy fee 11,300 sats, SegWit fee 7,050 sats, and savings 4,250 sats at 50 sat/vB.

## Evidence references

Evidence: terminal output from `cargo test --test lab_06`; tests verify weight, ceiling division, checked fee multiplication, and comparison.

## Explanation

Weight counts non-witness bytes at four units and witness bytes at one unit. Virtual size is weight divided by four and rounded up, so the result depends on transaction composition rather than being a universal flat discount.

