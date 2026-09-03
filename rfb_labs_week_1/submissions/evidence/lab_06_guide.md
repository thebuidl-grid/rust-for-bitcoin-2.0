# Lab 06: decodificación y conservación de valor

## Objetivo

Decodificar el pago pendiente y comprobar matemáticamente su comisión.

## Qué hace el código

`getrawtransaction <txid> 2` proporciona cada `vin.prevout.value`. El código identifica los inputs consumidos, outputs, el pago del receiver y el cambio; calcula la diferencia en precisión de satoshis.

## Explicación sencilla

Una transacción toma monedas anteriores y crea monedas nuevas. Lo que entra debe cubrir lo que sale. La pequeña diferencia no va a una dirección: queda disponible para el minero como fee.

## Pasos en Polar

1. Ejecutar `bitcoin-cli -regtest getrawtransaction <txid> 2`.
2. Anotar cada `vin` como `txid:vout` y su `prevout.value`; anotar `vout`, direcciones, valores y `vsize`.
3. Calcular `sum(inputs) - sum(outputs)` y registrar pago, cambio y fee.

## Resultado esperado

Debe cumplirse `sum(inputs) = pago + cambio + fee`. Un output `OP_RETURN` no se considera cambio porque no representa una moneda gastable normal.
