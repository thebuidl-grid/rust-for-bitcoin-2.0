#!/bin/bash
# Example 4: Custom version and locktime
# Shows how to use different version numbers and locktime values

cargo run -- \
  --version 1 \
  --locktime 500000 \
  --input '{"prev_txid":"8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821","vout":0,"script_sig":"","sequence":0,"witness":[]}' \
  --output '{"value":100000,"script_pubkey":"76a914a632c1fff47af29f8c81dc4c6e91eb49a116c12b88ac"}'
