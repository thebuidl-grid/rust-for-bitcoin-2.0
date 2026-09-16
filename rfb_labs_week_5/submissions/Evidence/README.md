# Evidence Documentation

This directory contains evidence and test results for all 10 labs in Week 5.

## Structure

Each lab has a corresponding subdirectory:
- `Lab_01/` - Address and network identification
- `Lab_02/` - Legacy P2PKH addresses
- `Lab_03/` - P2SH multisig
- `Lab_04/` - Native P2WPKH (SegWit)
- `Lab_05/` - Address compatibility
- `Lab_06/` - Transaction weight and fees
- `Lab_07/` - BIP39 mnemonics
- `Lab_08/` - BIP32 extended keys
- `Lab_09/` - BIP44 paths
- `Lab_10/` - Multi-standard address recovery

## Test Results Summary

All 40 functions pass their tests:
- **Lab 01-10**: 4/4 tests passing each = 40/40 total
- **Grader Score**: 40/70 (automated tests all passing)
- **Code Status**: Compiled, formatted (cargo fmt), tested

## Running Tests

To run tests for a specific lab:
```bash
cargo test --test lab_01
cargo test --test lab_02
# ... etc
```

To run all tests:
```bash
cargo test
```

To run the grader:
```bash
bash grader/grade.sh
```

## Evidence Types

### Rust Test Execution
Each lab's `.md` file includes:
- Commands used (cargo test --test labXX)
- Terminal output (test results)
- Evidence references (which functions passed)

### Explanations
Each lab includes a technical explanation of:
- Core concepts (address formats, key derivation, etc.)
- Practical implications (fees, compatibility, security)
- Real-world usage patterns

## Key Achievements

✅ **All 40 functions implemented**
- Lab 01: Address identification and validation (4 functions)
- Lab 02: P2PKH address construction (4 functions)
- Lab 03: P2SH multisig (4 functions)
- Lab 04: Native SegWit P2WPKH (4 functions)
- Lab 05: Address format compatibility (4 functions)
- Lab 06: Transaction weight and fees (4 functions)
- Lab 07: BIP39 mnemonic handling (4 functions)
- Lab 08: BIP32 extended key derivation (4 functions)
- Lab 09: BIP44 path decoding (4 functions)
- Lab 10: Multi-standard wallet recovery (4 functions)

✅ **All tests passing**
- Integration tests verify correct behavior
- Tests cover happy path and error cases
- Grader confirms all automated points

✅ **Code quality**
- Formatted with `cargo fmt`
- No compiler warnings
- Error handling with custom LabError type
