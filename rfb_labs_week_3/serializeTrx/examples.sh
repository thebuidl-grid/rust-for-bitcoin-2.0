#!/bin/bash

set -e

echo "==============================================="
echo "Bitcoin Transaction Serializer Examples"
echo "==============================================="

BINARY="cargo run --quiet --"

echo -e "\n\n1️⃣  EXAMPLE 1: Simple SegWit Transaction (1 input, 2 outputs)"
echo "=================================================="
$BINARY \
  --version 2 \
  --segwit \
  --locktime 0 \
  --inputs '[
    {
      "prev_txid": "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821",
      "vout": 1,
      "script_sig": "",
      "sequence": 4294967295,
      "witness": [
        "3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301",
        "029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358"
      ]
    }
  ]' \
  --outputs '[
    {
      "value": 69886,
      "script_pubkey": "0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"
    },
    {
      "value": 29442,
      "script_pubkey": "00149831122b93d21715c70db626ccc844d3c21f9687"
    }
  ]'

echo -e "\n\n2️⃣  EXAMPLE 2: Non-SegWit Legacy Transaction"
echo "=================================================="
$BINARY \
  --version 1 \
  --locktime 0 \
  --inputs '[
    {
      "prev_txid": "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821",
      "vout": 0,
      "script_sig": "483045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab30121029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358",
      "sequence": 4294967295
    }
  ]' \
  --outputs '[
    {
      "value": 99328,
      "script_pubkey": "76a914a632c1fff47af29f8c81dc4c6e91eb49a116c12b88ac"
    }
  ]'

echo -e "\n\n3️⃣  EXAMPLE 3: Transaction with Multiple Inputs"
echo "=================================================="
$BINARY \
  --version 2 \
  --segwit \
  --locktime 0 \
  --inputs '[
    {
      "prev_txid": "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821",
      "vout": 0,
      "script_sig": "",
      "sequence": 4294967295,
      "witness": [
        "3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301",
        "029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358"
      ]
    },
    {
      "prev_txid": "a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1",
      "vout": 1,
      "script_sig": "",
      "sequence": 4294967294,
      "witness": [
        "30440220f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301",
        "029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358"
      ]
    }
  ]' \
  --outputs '[
    {
      "value": 50000,
      "script_pubkey": "0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"
    },
    {
      "value": 25000,
      "script_pubkey": "00149831122b93d21715c70db626ccc844d3c21f9687"
    }
  ]'

echo -e "\n\n4️⃣  EXAMPLE 4: Empty Witness (SegWit without actual witness)"
echo "=================================================="
$BINARY \
  --version 2 \
  --segwit \
  --locktime 0 \
  --inputs '[
    {
      "prev_txid": "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821",
      "vout": 0,
      "script_sig": "",
      "sequence": 4294967295,
      "witness": []
    }
  ]' \
  --outputs '[
    {
      "value": 100000,
      "script_pubkey": "0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"
    }
  ]'

echo -e "\n\n5️⃣  EXAMPLE 5: Transaction with Locktime"
echo "=================================================="
$BINARY \
  --version 2 \
  --segwit \
  --locktime 800000 \
  --inputs '[
    {
      "prev_txid": "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821",
      "vout": 1,
      "script_sig": "",
      "sequence": 0,
      "witness": [
        "3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301",
        "029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358"
      ]
    }
  ]' \
  --outputs '[
    {
      "value": 50000,
      "script_pubkey": "0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"
    }
  ]'

echo -e "\n\n6️⃣  ERROR EXAMPLE: Invalid Hex in Transaction ID"
echo "=================================================="
echo "Running with invalid hex (should fail gracefully)..."
$BINARY \
  --version 2 \
  --segwit \
  --locktime 0 \
  --inputs '[
    {
      "prev_txid": "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ",
      "vout": 0,
      "script_sig": "",
      "sequence": 4294967295,
      "witness": []
    }
  ]' \
  --outputs '[{"value": 100000, "script_pubkey": "0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"}]' \
  || echo "✓ Error handled correctly (invalid hex detected)"

echo -e "\n\n7️⃣  ERROR EXAMPLE: Odd-length Hex String"
echo "=================================================="
echo "Running with odd-length hex (should fail gracefully)..."
$BINARY \
  --version 2 \
  --segwit \
  --locktime 0 \
  --inputs '[
    {
      "prev_txid": "abc",
      "vout": 0,
      "script_sig": "",
      "sequence": 4294967295,
      "witness": []
    }
  ]' \
  --outputs '[{"value": 100000, "script_pubkey": "0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"}]' \
  || echo "✓ Error handled correctly (odd-length hex detected)"

echo -e "\n\n==============================================="
echo "✓ All examples completed successfully!"
echo "==============================================="
