# Lab 05: broadcast y mempool

## Objetivo

Enviar exactamente 1 BTC sin minar y demostrar que la transacción fue difundida, pero no confirmada.

## Qué hace el código

`send_btc` usa `sendtoaddress`; el TXID se busca en `getrawmempool`; `get_transaction_status` extrae confirmaciones, importe, comisión y bloque; el receiver se inspecciona con `getbalances`.

## Explicación sencilla

Firmar crea una autorización; transmitir la comparte con nodos; el mempool es la sala de espera; confirmar significa que un minero la incluyó en un bloque. Estar en la sala de espera no equivale a estar escrita en la cadena.

## Pasos en Polar

1. Ejecutar `-rpcwallet=miner sendtoaddress <receiver_address> 1` y guardar el TXID.
2. Ejecutar `getrawmempool`, `-rpcwallet=miner gettransaction <txid>` y `-rpcwallet=receiver getbalances` sin minar.
3. Guardar la salida donde el TXID está en mempool, las confirmaciones son 0 y el receiver tiene saldo pendiente.

## Resultado esperado

El TXID aparece localmente, el sender informa cero confirmaciones y el receiver muestra `untrusted_pending` de 1 BTC.
