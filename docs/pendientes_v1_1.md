# Pendientes para v1.1

## P3 — TipoRegimen como campo de `CfdiRequest`

`commands/cfdi_cmd.rs` mantiene `TipoRegimen="02"` hardcodeado en el nodo
`nomina12:Receptor`. Valores válidos según el catálogo `c_TipoRegimen` del SAT:

- `02` — Sueldos y Salarios e Ingresos Asimilados a Salarios
- `03` — Jubilados
- `04` — Pensionados
- `05` — Asimilados a Salarios (Miembros de las Sociedades Cooperativas de Producción)

Para v1.1 debe agregarse `tipo_regimen: String` a `CfdiRequest`, validarse contra el
catálogo y permitir al usuario seleccionarlo en la configuración del trabajador.

## P4 — FechaInicioRelLaboral y Antigüedad

`FechaInicioRelLaboral` está hardcodeado a `"2020-01-01"` y `Antigüedad` a `"P1W"`.
Deben derivarse del trabajador:

- `FechaInicioRelLaboral`: campo ISO 8601 `AAAA-MM-DD` en `CfdiRequest`, tomado de
  `trabajadores.fecha_ingreso`.
- `Antigüedad`: duración ISO 8601 calculada entre `fecha_ingreso` y `fecha_pago`.
  Formato `PnYnMnD` o `PnW`. Ejemplo: 2 años, 3 meses y 15 días → `P2Y3M15D`.

También debe validarse contra las reglas de diferenciación de percepciones por
antigüedad del Anexo 20 RMF.
