# Lab 02 — Wallets and addresses

## Commands used

TODO: Record how you created and inspected both wallets and addresses.

`cargo test --test lab_02` : This command runs the test to verify `create_wallet`, `list_wallets`, `get_new_address`, and `address_belongs_to_wallet`

`bitcoin-cli -regtest createwallet` "test": for creating a wallet with the name `test`

`bitcoin-cli -regtest listwallets` — confirmed the wallet was loaded.
`bitcoin-cli -regtest -rpcwallet=test getnewaddress "demo"` — generated a labelled address inside the `test` wallet.
`bitcoin-cli -regtest -rpcwallet=test getaddressinfo <address>` — confirmed the wallet controls the generated address (`ismine: true`).

## Terminal output

TODO: Include loaded wallets, addresses, and ownership evidence.

<img width="1034" height="437" alt="Screenshot 2026-07-31 at 23 25 24" src="https://github.com/user-attachments/assets/29934be5-57aa-4a8c-b1f1-5b2cd1c7a9aa" />


## Evidence references

TODO: Link screenshots or describe the attached evidence.
The screenshot above shows `ismine": true,` confirming ownership, hence the owner has the ability to spend funds sent to it. it also contains the address, timestamp, and many more

## Explanation

TODO: Explain wallet context and the purpose of `-rpcwallet`.

Wallet context determines which loaded wallet a Bitcoin Core RPC command operates on (balance, addresses, transactions, etc.).



`-rpcwallet` explicitly selects that wallet for the RPC request, which is required when multiple wallets are loaded.
