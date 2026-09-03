# Lab 10: ramas competidoras y reorg

## Objetivo

Separar dos nodos, minar ramas privadas de longitudes distintas y observar la convergencia tras reconectarlos.

## Qué hace el código

`get_chain_tip` lee altura, hash punta y `chainwork`; `disconnect_peer` y `reconnect_peer` llaman los RPC de red. `build_reorg_report` informa convergencia sólo cuando hash y altura finales coinciden.

## Explicación sencilla

Dos nodos pueden ver temporalmente historias distintas. Al volver a hablar, ambos validan las reglas y siguen la cadena con mayor trabajo acumulado. La rama descartada queda stale; el cambio de una historia aceptada por otra mejor se llama reorganización.

## Pasos en Polar

1. Añadir un segundo nodo, conectarlo y registrar `getblockchaininfo` en ambos hasta obtener el mismo hash y altura.
2. Desconectarlos con `disconnectnode`; minar 2 bloques en A y 4 en B. Guardar puntas y `chainwork` de ambos.
3. Reconectar con `addnode <peer> onetry`, esperar sincronización y registrar de nuevo ambas puntas.

## Resultado esperado

Ambos nodos terminan con la punta de mayor trabajo acumulado. La decisión no depende de qué minero la hizo ni de cuál mensaje llegó primero.
