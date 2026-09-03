# Lab 04: UTXOs y outpoints

## Objetivo

Inspeccionar las salidas no gastadas del miner, elegir una gastable y reconciliar su suma con el balance.

## Qué hace el código

`list_unspent` convierte cada respuesta de `listunspent` a `Utxo`; `select_spendable_utxo` prefiere más confirmaciones; `outpoint` produce el par único `txid:vout`; la suma descarta salidas no gastables.

## Explicación sencilla

Bitcoin no lleva un saldo como una sola cifra guardada. Una wallet controla varias "monedas" separadas (UTXOs). Un outpoint es la coordenada exacta de una de ellas: transacción que la creó más su posición dentro de esa transacción.

## Pasos en Polar

1. Ejecutar `bitcoin-cli -regtest -rpcwallet=miner listunspent`.
2. Registrar para una salida gastable `txid`, `vout`, `amount`, `confirmations`, `address`, `scriptPubKey` y `spendable`.
3. Sumar los `amount` de las entradas con `spendable: true` y compararlo con `getbalances`.

## Resultado esperado

La suma de UTXOs gastables explica el balance disponible; no es un asiento contable independiente.
