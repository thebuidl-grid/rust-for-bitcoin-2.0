# Lab 07: confirmación y bloque contenedor

## Objetivo

Minar un bloque y demostrar que el pago salió del mempool y pertenece a ese bloque.

## Qué hace el código

Se mina exactamente un bloque con `generatetoaddress`, se consulta el mempool, la wallet obtiene `confirmations` y `blockhash`, y `getblock <hash> 1` se usa para buscar el TXID en `tx`.

## Explicación sencilla

La misma transacción no cambia al confirmarse: su contenido y TXID se mantienen. Lo nuevo es que queda incluida en un bloque aceptado por la cadena, obteniendo una posición en el historial compartido.

## Pasos en Polar

1. Ejecutar `generatetoaddress 1 <miner_address>`.
2. Ejecutar `getrawmempool` y `-rpcwallet=receiver gettransaction <txid>`.
3. Con el `blockhash` obtenido, ejecutar `getblock <blockhash> 1` y localizar el TXID.

## Resultado esperado

El mempool queda vacío, la transacción tiene una confirmación, contiene un hash de bloque y ese bloque lista el mismo TXID.
