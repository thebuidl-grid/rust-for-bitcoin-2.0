# Lab 03: madurez de coinbase

## Objetivo

Mostrar que una recompensa de minería no se puede gastar hasta tener 100 confirmaciones.

## Qué hace el código

El flujo mina un bloque, lee `getbalances`, intenta enviar 1 BTC y conserva el error RPC. Después mina 100 bloques adicionales y vuelve a registrar altura y balances.

## Explicación sencilla

La moneda creada al minar no está disponible inmediatamente: Bitcoin espera 100 bloques para reducir problemas si la cadena se reorganiza. En una cadena nueva, el primer bloque queda gastable a altura 101.

## Pasos en Polar

1. Obtener una dirección del miner y una del receiver.
2. Ejecutar `generatetoaddress 1 <miner_address>`, `-rpcwallet=miner getbalances` y el intento `sendtoaddress <receiver_address> 1`.
3. Conservar el error de fondos insuficientes; ejecutar `generatetoaddress 100 <miner_address>` y repetir altura y balances.

## Resultado esperado

A altura 1 la recompensa figura como `immature`; a altura 101 la primera recompensa aparece como `trusted`, mientras recompensas posteriores siguen inmaduras.
