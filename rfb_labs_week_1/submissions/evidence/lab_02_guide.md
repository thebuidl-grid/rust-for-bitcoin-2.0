# Lab 02: wallets y direcciones

## Objetivo

Crear las wallets `miner` y `receiver`, y generar las direcciones etiquetadas `mining` y `classmate`.

## Qué hace el código

`create_wallet` usa `createwallet`; `list_wallets` decodifica el arreglo de `listwallets`; las consultas de dirección se hacen con el contexto de wallet (`-rpcwallet`), que el cliente añade al recibir `Some(wallet_name)`.

## Explicación sencilla

Una wallet administra claves y conoce qué monedas le pertenecen. Una dirección es un destino público para recibir. Usar la wallet equivocada es como consultar el inventario de otra caja fuerte: la dirección puede existir, pero esa wallet no la controla.

## Pasos en Polar

1. Ejecutar `bitcoin-cli -regtest createwallet miner` y `bitcoin-cli -regtest createwallet receiver`.
2. Ejecutar `bitcoin-cli -regtest -rpcwallet=miner getnewaddress mining` y el equivalente para `receiver classmate`.
3. Verificar `listwallets` y `getaddressinfo <direccion>` en cada wallet. Las direcciones deben comenzar con `bcrt1`.

## Resultado esperado

Las dos wallets aparecen cargadas y `ismine` es `true` únicamente en la wallet que generó cada dirección.
