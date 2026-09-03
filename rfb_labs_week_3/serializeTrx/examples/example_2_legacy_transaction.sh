#!/bin/bash
# Example 2: Legacy (non-SegWit) transaction
# A simple transaction without witness data or SegWit marker

cargo run -- \
  --version 2 \
  --locktime 0 \
  --input '{"prev_txid":"8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821","vout":0,"script_sig":"","sequence":4294967295,"witness":[]}' \
  --output '{"value":69886,"script_pubkey":"0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"}'
