# Lab 09: selección de múltiples UTXOs

## Objetivo

Entregar a Alice tres UTXOs de 0.4 BTC y comprobar que necesita combinar más de uno para pagar 1 BTC.

## Qué hace el código

El miner emite tres pagos separados. Tras confirmarlos, `listunspent` filtra las salidas de Alice; su pago de 1 BTC se decodifica reutilizando Lab 06 para contar inputs, distinguir pago/cambio y calcular la comisión.

## Explicación sencilla

Tres monedas de 0.4 BTC suman 1.2 BTC, así que ninguna por sí sola alcanza 1 BTC. La wallet junta varias monedas, paga y devuelve el sobrante como cambio. Esa combinación puede revelar que las entradas pertenecen a la misma persona.

## Pasos en Polar

1. Crear wallet y dirección Alice; enviar tres veces `sendtoaddress <alice_address> 0.4` desde miner.
2. Minar un bloque y verificar los tres UTXOs mediante `-rpcwallet=alice listunspent`.
3. Enviar 1 BTC de Alice al receiver, decodificar con `getrawtransaction <txid> 2` y registrar inputs, output de pago, cambio y fee.

## Resultado esperado

El gasto usa al menos dos inputs, entrega 1 BTC al receiver y deja cambio más una comisión. Combinar inputs tiene un costo de privacidad por la heurística de propiedad común.
