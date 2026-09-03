#!/bin/bash
# Example 3: Multi-input, multi-output SegWit transaction
# Demonstrates a more complex transaction with multiple inputs feeding into multiple outputs

cargo run -- \
  --version 2 \
  --segwit \
  --locktime 0 \
  --input '{"prev_txid":"8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821","vout":0,"script_sig":"","sequence":4294967295,"witness":[]}' \
  --input '{"prev_txid":"1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef","vout":1,"script_sig":"","sequence":4294967295,"witness":[]}' \
  --output '{"value":50000,"script_pubkey":"0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"}' \
  --output '{"value":40000,"script_pubkey":"00149831122b93d21715c70db626ccc844d3c21f9687"}' \
  --output '{"value":10000,"script_pubkey":"0014aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}'
