# Lab 06 — Transaction decoding

## Commands used

TODO: Record the verbose transaction-decoding commands.

`cargo test --test lab_06`- ran the Lab 06 test 

`bitcoin-cli -regtest getrawtransaction <txid> 2`: `txid` is the one obtained from `bitcoin-cli -regtest -rpcwallet=test sendtoaddress <receiver_address> 1`, where 2 is the verbosity level 

`bitcoin-cli -regtest decoderawtransaction <raw_hex>` - hex value is obtained from the command above, i.e., `bitcoin-cli -regtest getrawtransaction <txid> 2`

## Terminal output

TODO: Include vin, vout, addresses, values, vsize, and calculated fee.
<img width="1123" height="535" alt="Screenshot 2026-08-02 at 11 38 02" src="https://github.com/user-attachments/assets/16e77f2a-5869-4fd3-8b65-0c03737d5061" />
<img width="1124" height="420" alt="Screenshot 2026-08-02 at 11 40 28" src="https://github.com/user-attachments/assets/fb88cec9-c7f4-4845-9bc8-5ac9e0f4c044" />
<img width="1137" height="750" alt="Screenshot 2026-08-02 at 11 41 21" src="https://github.com/user-attachments/assets/9f06712e-811a-4f2b-9a95-97ec0cf7cbe1" />
<img width="669" height="183" alt="Screenshot 2026-08-02 at 11 42 53" src="https://github.com/user-attachments/assets/5d740418-0d20-4966-a88a-59c7b36903c7" />

## Evidence references

TODO: Link screenshots or describe the attached evidence.

`getrawtransaction 2` fetches the transaction from the node and shows a **verbose decoded view** (plus extra blockchain context when available). 

`decoderawtransaction ` takes the **raw hex string** and decodes it locally. It describes the **same transaction**, which is why the `txid`, inputs, outputs, `vsize`, and addresses match.

What this transaction is doing: - It spends **two previous outputs** (`0adcee...:0` and `f535b4...:0`) — those are the inputs. - It creates **two new outputs**: - `1.00000000 BTC` to the **receiver** address `bcrt1qutw99...` - `0.99999792 BTC` back to a **change address** owned by the sender. - The missing amount is the miner fee:
2.00000000−(1.00000000+0.99999792)=0.00000208 BTC

`vsize = 208` means the transaction occupies **208 virtual bytes**, which is used for fee calculation. 

## Explanation

TODO: Prove value conservation and explain why the fee has no dedicated output.

The transaction above spends two previous UTXOs totaling 2 BTC and creates two new UTXOs totaling 1.99999859 BTC. The remaining 0.00000141 BTC is the miner's fee. Bitcoin transactions do not create a dedicated “fee output”; instead, the fee is implicitly defined as sum(inputs) − sum(outputs). Miners claim this difference when they include the transaction in a block. This demonstrates Bitcoin’s value-conservation rule: every satoshi in the inputs is accounted for either in a new output or in the transaction fee.
