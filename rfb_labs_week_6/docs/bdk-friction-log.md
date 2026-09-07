# BDK Friction and Contribution Log

This document records friction encountered while building Mufasa. An entry is
not considered a BDK defect until we reproduce it independently, rule out
application misuse, and check upstream discussions.

## What to Record

- Confusing APIs or error messages
- Missing end-to-end examples
- Persistence and recovery difficulties
- Unexpected synchronization or reorganization behavior
- Poorly documented edge cases
- Missing regression, integration, property, or fuzz tests
- Compatibility friction between BDK crates

## Contribution Workflow

1. Reproduce the behavior in Mufasa.
2. Reduce it to a minimal standalone example.
3. Verify that configuration or application misuse is not the cause.
4. Search BDK issues, pull requests, documentation, and release notes.
5. Ask maintainers whether the behavior is intentional when unclear.
6. Add a failing test or concrete documentation example.
7. Propose one focused fix with its rationale.

## Evaluation Checklist

| Area | Question |
|---|---|
| Correctness | Which invariants must always hold? |
| Failure recovery | What happens when an operation stops halfway? |
| Security | Can secrets leak or adversarial input be accepted? |
| Concurrency | Can simultaneous operations corrupt or overwrite state? |
| Compatibility | What changes across crate, node, and database versions? |
| Performance | What degrades with a large wallet history? |
| API design | Which valid workflows are difficult or error-prone? |
| Documentation | Can a new user complete a realistic workflow? |
| Observability | Are errors specific and diagnosable? |
| Testing | Which meaningful behaviors lack coverage? |

## Candidate Observations

### 2026-09-07 — Minimal SQLite persistence lifecycle

- **Component:** `bdk_wallet` 3.1 and `bdk_sqlite` 0.6
- **Status:** Investigating; not classified as a bug
- **Observation:** We need a clear create, load, mutate, persist, and reopen
  lifecycle for an asynchronous SQLite-backed descriptor wallet.
- **Question:** Is there one current end-to-end example covering this complete
  lifecycle, including failure handling?
- **Next evidence:** Implement Mufasa's persistence flow, record the exact API
  friction, and compare it with the official examples before opening an issue.

## Entry Template

### YYYY-MM-DD — Short title

- **Component and version:**
- **Status:** Observed / Reproduced / Upstream discussion / Proposed / Closed
- **Context:**
- **Expected behavior:**
- **Actual behavior:**
- **Minimal reproduction:**
- **Why this may be application misuse:**
- **Existing upstream issue or documentation:**
- **Potential test or improvement:**
- **Next action:**
