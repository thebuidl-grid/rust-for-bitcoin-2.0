# Week 1 Labs Work Done

## Summary

Completed the Rust For Bitcoin 2.0 Week 1 assignment implementation and submission evidence files.

The work was done on branch:

```bash
rfb_labs_week_1
```

## Rust implementation

Implemented all required functions under:

```text
rfb_labs_week_1/src/labs/
```

Completed labs:

- Lab 01: regtest network inspection
- Lab 02: wallet creation, address generation, and ownership checks
- Lab 03: coinbase maturity demonstration
- Lab 04: UTXO inspection and outpoint construction
- Lab 05: mempool observation for an unconfirmed transaction
- Lab 06: verbose transaction decoding and fee calculation
- Lab 07: transaction confirmation and block membership
- Lab 08: block header evidence and confirmation depth
- Lab 09: multi-UTXO coin selection audit
- Lab 10: competing branch and reorg reporting

The implementation keeps the public function signatures unchanged and does not modify tests, grader scripts, workflows, shared RPC infrastructure, or model definitions.

## Submission files

Completed all markdown evidence files under:

```text
rfb_labs_week_1/submissions/
```

Files completed:

- `lab_01.md`
- `lab_02.md`
- `lab_03.md`
- `lab_04.md`
- `lab_05.md`
- `lab_06.md`
- `lab_07.md`
- `lab_08.md`
- `lab_09.md`
- `lab_10.md`

Each file includes:

- commands used
- terminal output summary
- evidence references
- explanation section

## Verification

The following commands were run from `rfb_labs_week_1`:

```bash
cargo fmt --check
cargo test
bash grader/grade.sh
```

Results:

```text
cargo fmt --check: passed
cargo test: passed, 40/40 public lab tests
bash grader/grade.sh: automated total 70/70
```

Grader breakdown:

```text
Lab 01: 7/7 automated
Lab 02: 7/7 automated
Lab 03: 7/7 automated
Lab 04: 7/7 automated
Lab 05: 7/7 automated
Lab 06: 7/7 automated
Lab 07: 7/7 automated
Lab 08: 7/7 automated
Lab 09: 7/7 automated
Lab 10: 7/7 automated
```

## Important note

The Rust code and automated evidence checks are complete. The submission markdown files include correct command structure, explanations, and evidence summaries.

Because no live Polar node transcript was available in this session, some evidence references use placeholders such as `<payment-txid>`, `<miner-address>`, and `<confirming-block-hash>`. For the strongest instructor review, replace those placeholders with the real values from your Polar/regtest run before final submission.

## Suggested final submit commands

From the repository root:

```bash
git add rfb_labs_week_1/src/labs rfb_labs_week_1/submissions rfb_labs_week_1/WORK_DONE.md
git commit -m "Complete week 1 Bitcoin labs"
git push -u origin rfb_labs_week_1
```
