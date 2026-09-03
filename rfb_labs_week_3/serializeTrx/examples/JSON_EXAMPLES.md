# JSON Input File Examples

## Simple Input (input.json)

```json
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
```

## Simple Output (output.json)

```json
{
  "value": 69886,
  "script_pubkey": "0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"
}
```

## Using JSON Files with the Tool

### Using with bash substitution:

```bash
cargo run -- \
  --version 2 \
  --segwit \
  --input "$(cat input.json)" \
  --output "$(cat output.json)"
```

### Using with jq (requires jq to be installed):

```bash
cargo run -- \
  --version 2 \
  --segwit \
  --input "$(jq -c . input.json)" \
  --output "$(jq -c . output.json)"
```

## Multiple Inputs Example (inputs.json)

```json
{
  "inputs": [
    {
      "prev_txid": "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821",
      "vout": 0,
      "script_sig": "",
      "sequence": 4294967295,
      "witness": []
    },
    {
      "prev_txid": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
      "vout": 1,
      "script_sig": "",
      "sequence": 4294967295,
      "witness": []
    }
  ]
}
```

### Using multiple inputs from a file:

```bash
cargo run -- \
  --version 2 \
  --segwit \
  --input "$(jq -c '.inputs[0]' inputs.json)" \
  --input "$(jq -c '.inputs[1]' inputs.json)" \
  --output '{"value":50000,"script_pubkey":"0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b"}' \
  --output '{"value":40000,"script_pubkey":"00149831122b93d21715c70db626ccc844d3c21f9687"}'
```
