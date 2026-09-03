# Lab 01: red regtest

## Objetivo

Verificar que Bitcoin Core está ejecutándose en `regtest`, y registrar su altura y bloque punta.

## Qué hace el código

`get_chain`, `get_block_height` y `get_best_block_hash` llaman a `getblockchaininfo`, `getblockcount` y `getbestblockhash`. `inspect_network` rechaza una cadena distinta de `regtest` para evitar operar por accidente en mainnet.

## Explicación sencilla

Polar crea y administra el escenario; Docker ejecuta los contenedores; Bitcoin Core valida la cadena; regtest es una cadena privada donde se pueden minar bloques al instante. La altura cuenta bloques y el hash punta identifica el último bloque.

## Pasos en Polar

1. Crear una red llamada `Week 1 Bitcoin Fundamentals` con un nodo Bitcoin Core y arrancarla.
2. En su terminal ejecutar `bitcoin-cli -regtest getblockchaininfo`, `bitcoin-cli -regtest getblockcount` y `bitcoin-cli -regtest getbestblockhash`.
3. Guardar la salida real y una captura; debe indicar `"chain": "regtest"`.

## Resultado esperado

La función devuelve `NetworkSnapshot { chain, block_height, best_block_hash }`. La altura y el hash cambian al minar, por eso la evidencia debe ser capturada en la red del estudiante.
