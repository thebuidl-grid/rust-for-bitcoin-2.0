# Lab 08: encabezados, proof of work y profundidad

## Objetivo

Registrar las propiedades del bloque confirmador y llevar una confirmación hasta seis.

## Qué hace el código

El código decodifica `getblockheader`: hash previo, raíz de Merkle, nonce, bits, dificultad, confirmaciones y chainwork. Después mina cinco bloques y vuelve a leer las confirmaciones de la transacción.

## Explicación sencilla

Cada bloque apunta al anterior, formando una historia enlazada. La raíz de Merkle resume todas sus transacciones. El proof of work requiere probar nonces hasta cumplir el objetivo; añadir bloques hace más costoso reemplazar ese historial.

## Pasos en Polar

1. Ejecutar `getblockheader <blockhash>` y guardar todos los campos solicitados.
2. Consultar `-rpcwallet=receiver gettransaction <txid>` para observar 1 confirmación.
3. Ejecutar `generatetoaddress 5 <miner_address>` y repetir `gettransaction`.

## Resultado esperado

La transacción llega a 6 confirmaciones. Más confirmaciones elevan el costo de una reorganización, pero no convierten una transacción inválida en válida.
