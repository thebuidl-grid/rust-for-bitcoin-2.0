# Lab 06 — Weight, virtual size, and fees

## Commands used

```bash
cargo test --test lab_06
```

## Terminal output

```bash
running 4 tests
test calculates_bip141_weight ... ok
test calculates_fee_from_feerate ... ok
test rounds_weight_up_to_virtual_bytes ... ok
test reproduces_the_class_fee_comparison ... ok

test result: ok. 4 passed; 0 failed
```

## Evidence references

All 4 tests pass, confirming:
- `transaction_weight()`: Calculates weight = (stripped_size × 3) + total_size
- `virtual_size()`: Computes vsize = ceil(weight / 4) = (weight + 3) / 4
- `fee_sats()`: Multiplies vbytes × feerate with overflow checking
- `compare_fees()`: Compares legacy vs. SegWit fees at same rate

## Explanation

BIP141 (SegWit) introduced "weight" to separate transaction data into two categories based on how much blockchain space they consume.

**Why separate weight from size?**

Before SegWit, transaction size was straightforward: count every byte equally. But SegWit moved signature data into a separate witness section that older nodes don't validate, so it's "cheaper" in terms of block capacity.

**Weight formula:**
```
weight = (stripped_size × 3) + total_size
```

- **stripped_size**: Transaction with witness data removed (legacy-compatible size)
- **total_size**: Complete transaction including witness
- **Multiplying by 3**: Legacy data counts 4x, witness counts 1x
  - This ensures `weight ÷ 4 = virtual_size` correctly

**Example:**
```
Legacy tx:     stripped=150, total=150 → weight = (150×3) + 150 = 600 → vsize = 150
SegWit tx:     stripped=100, total=134 → weight = (100×3) + 134 = 434 → vsize = 108.5 ≈ 109
```
The SegWit transaction saves 41 bytes of virtual size (27% savings).

**Fee calculation:**
```
fee = virtual_size × feerate_sat/vb
```

Miners order transactions by feerate (sats per virtual byte), not per physical byte. This means SegWit users pay less in fees without the protocol sacrificing fairness.

**Key insight:**
SegWit didn't introduce a "discount" on witness data—it just changed how blocks count transaction data. Witness data still takes up space on the blockchain; it's just not part of the legacy transaction format, so older nodes skip validating it. The 4:1 weight ratio fairly reflects this distinction.

**Fee savings at 1 sat/vb:**
- Legacy tx (150 bytes): 150 sats
- SegWit tx (109 vbytes): 109 sats
- Savings: ~27%

