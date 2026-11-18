# Casos de validación contable — NominaMX v0.9

**Objetivo:** que el contador compare, caso por caso, lo que calcula NominaMX
contra SUA o con su cálculo manual, y deje constancia de la revisión.

**Instrucciones:** en cada caso llene la columna **Cifra que devuelve la app**
con el importe que ve en pantalla y marque **Resultado** con ✅ si coincide con
la cifra esperada (tolerancia de $0.02 por redondeo) o ❌ si no coincide. Si es
❌, escriba la diferencia en **Notas**. Al final firme.

## Supuestos comunes

| Concepto | Valor usado |
|----------|-------------|
| Periodicidad y días | Mensual, 30 días |
| Fecha de pago | 2026-02-28, salvo el caso 10 (2026-01-31) |
| UMA diaria | Enero $113.14; febrero en adelante $117.31 |
| Salario mínimo general 2026 | $278.80 |
| Subsidio al empleo | Enero $536.21; febrero en adelante $535.65; tope $11,492.66 |
| ISN | 3% (CDMX) |
| Prima de riesgo de trabajo | 0.5% |
| Factor de integración SDI | 1.0496528429 (15 días aguinaldo, 12 vacaciones, 25% prima) |
| IMSS | Cuotas según art. 29 LSS, redondeadas por concepto |
| INFONAVIT | Aportación patronal 5%; descuento tipo MONTO = factor × días |

---

## Caso 1 — Salario mínimo, mes completo, sin INFONAVIT

**Datos de entrada:** salario diario $278.80; 30 días; INFONAVIT inactivo.

**Cálculo manual:** SBC = 278.80 × 1.0496528429 = 292.6432; SBC total =
8,779.2964. ISR sobre 8,364.00: 420.95 + (8,364.00 − 7,168.52) × 10.88% =
551.018224; menos subsidio 535.65 = **15.368224**. IMSS obrero 208.51.

| Concepto | Cifra esperada | Cifra que devuelve la app | Resultado |
|----------|---------------:|--------------------------:|-----------|
| Percepciones | 8,364.00 | | |
| ISR | 15.37 | | |
| IMSS obrero | 208.51 | | |
| Deducciones | 223.88 | | |
| Neto | **8,140.12** | | |
| Costo patronal | 2,707.70 | | |

**Notas:** ________________________________________________________

---

## Caso 2 — $15,000 al mes, sin INFONAVIT

**Datos de entrada:** salario diario $500.00; 30 días; INFONAVIT inactivo.

**Cálculo manual:** SBC = 500 × 1.0496528429 = 524.8264; ISR sobre 15,000:
1,339.14 + (15,000 − 14,644.65) × 17.92% = 1,402.81872. El subsidio es 0 porque
15,000 > 11,492.66. IMSS obrero 394.68 (incluye CEAV patronal de 20.75).

| Concepto | Cifra esperada | Cifra que devuelve la app | Resultado |
|----------|---------------:|--------------------------:|-----------|
| Percepciones | 15,000.00 | | |
| ISR | 1,402.82 | | |
| IMSS obrero | 394.68 | | |
| Deducciones | 1,797.50 | | |
| Neto | **13,202.50** | | |
| Costo patronal | 4,651.55 | | |

**Notas:** ________________________________________________________

---

## Caso 3 — $15,000 al mes, con INFONAVIT factor 1.0

**Datos de entrada:** salario diario $500.00; 30 días; INFONAVIT activo, tipo
MONTO, factor 1.0.

**Cálculo manual:** mismo ISR e IMSS que el caso 2. Descuento INFONAVIT =
1.0 × 30 = 30.00 (tope 278.80 × 20% × 30 = 1,672.80). Deducciones =
1,402.81872 + 394.68 + 30.00 = 1,827.49872.

| Concepto | Cifra esperada | Cifra que devuelve la app | Resultado |
|----------|---------------:|--------------------------:|-----------|
| Percepciones | 15,000.00 | | |
| ISR | 1,402.82 | | |
| IMSS obrero | 394.68 | | |
| Descuento INFONAVIT | 30.00 | | |
| Deducciones | 1,827.50 | | |
| Neto | **13,172.50** | | |
| Costo patronal | 4,651.55 | | |

**Notas:** Si el tipo de descuento es CUOTA_FIJA_VSM en vez de MONTO, el
descuento cambia; verifique el tipo en el alta del trabajador.
__________________________________________________________________

---

## Caso 4 — Salario alto: 5 UMA, con excedente CEAV

**Datos de entrada:** salario diario 5 × 117.31 = $586.55; 30 días; INFONAVIT
inactivo.

**Cálculo manual:** SBC = 586.55 × 1.0496528429 = 615.6739. CEAV =
(615.6739 − 3 × 117.31) × 30 × 0.004 = 31.65. ISR sobre 17,596.50 =
1,856.84 + (17,596.50 − 17,533.65) × 21.36% = 1,870.26476.

| Concepto | Cifra esperada | Cifra que devuelve la app | Resultado |
|----------|---------------:|--------------------------:|-----------|
| Percepciones | 17,596.50 | | |
| ISR | 1,870.26 | | |
| IMSS obrero | 470.32 | | |
| CEAV (parte patronal) | 31.65 | | |
| Deducciones | 2,340.58 | | |
| Neto | **15,255.92** | | |
| Costo patronal | 5,352.54 | | |

**Notas:** La cifra de CEAV de los tests unitarios (28.15) usa el salario
directo sin integrar; en el recibo se usa el SBC integrado, por eso es 31.65.
__________________________________________________________________

---

## Caso 5 — Incapacidad de 5 días, verificar IMSS

**Datos de entrada:** salario diario $586.55; 30 días cotizados; 5 días de
incapacidad; UMA $117.31.

**Cálculo manual:** el motor calcula las cuotas sobre los 30 días cotizados; los
días de incapacidad no modifican las cuotas (IMSS obrero 446.07, patronal
3,393.14). Diferencia contra 0 días de incapacidad: 0.00.

| Concepto | Cifra esperada | Cifra que devuelve la app | Resultado |
|----------|---------------:|--------------------------:|-----------|
| IMSS obrero | 446.07 | | |
| IMSS patronal | 3,393.14 | | |
| Diferencia por incapacidad | **0.00** | | |
| Descuento INFONAVIT (si aplica) | 30.00 | | |

**Notas:** Este caso solo valida cuotas. La prestación económica de la
incapacidad (subsidio del IMSS o pago patronal) NO está implementada en v0.9;
no hay pantalla ni comando para calcularla.
__________________________________________________________________

---

## Caso 6 — Finiquito por renuncia, 2 años de antigüedad

**Datos de entrada:** salario diario $278.80; 5 días de salario pendientes;
12 días de vacaciones pendientes; prima vacacional 25%; 180 días trabajados en
el año; mínimo de aguinaldo 15 días.

**Cálculo manual:** salario pendiente 1,394.00 + vacaciones 3,345.60 + prima
836.40 + aguinaldo proporcional 278.80 × (15 × 180 / 365) = 2,062.356164.

| Concepto | Cifra esperada | Cifra que devuelve la app | Resultado |
|----------|---------------:|--------------------------:|-----------|
| Días de salario pendientes | 1,394.00 | | |
| Vacaciones | 3,345.60 | | |
| Prima vacacional | 836.40 | | |
| Aguinaldo proporcional | 2,062.36 | | |
| Total finiquito | **7,638.36** | | |

**Notas:** Si se usan 14 días de vacaciones (tabla del art. 76 para 2 años), el
total sería 8,335.36. La cifra esperada usa 12 días, que es el vector canónico
del proyecto.
__________________________________________________________________

---

## Caso 7 — Liquidación por despido, 5 años de antigüedad

**Datos de entrada:** salario diario $278.80; finiquito base $5,000.00; 5 años
de antigüedad; salario mínimo $278.80.

**Cálculo manual:** tres meses 278.80 × 90 = 25,092.00; 20 días por año
278.80 × 20 × 5 = 27,880.00; prima de antigüedad 278.80 × 12 × 5 = 16,728.00
(no se topa porque 278.80 < 2 × 278.80).

| Concepto | Cifra esperada | Cifra que devuelve la app | Resultado |
|----------|---------------:|--------------------------:|-----------|
| Finiquito base | 5,000.00 | | |
| Tres meses | 25,092.00 | | |
| 20 días por año | 27,880.00 | | |
| Prima de antigüedad | 16,728.00 | | |
| Total liquidación | **74,700.00** | | |

**Notas:** ________________________________________________________

---

## Caso 8 — Aguinaldo proporcional por 180 días

**Datos de entrada:** salario diario $278.80; 180 días trabajados; 15 días
mínimos de aguinaldo (art. 87 LFT).

**Cálculo manual:** 278.80 × (15 × 180 / 365) = 278.80 × 2,700 / 365 =
2,062.356164.

| Concepto | Cifra esperada | Cifra que devuelve la app | Resultado |
|----------|---------------:|--------------------------:|-----------|
| Aguinaldo proporcional | **2,062.36** | | |

**Notas:** Este importe es el mismo componente de aguinaldo del caso 6.
__________________________________________________________________

---

## Caso 9 — Prima vacacional, año 3

**Datos de entrada:** salario diario $278.80; 3 años de antigüedad (16 días de
vacaciones según tabla del art. 76 LFT); prima 25% (art. 80 LFT).

**Cálculo manual:** 278.80 × 16 × 0.25 = 1,115.20.

| Concepto | Cifra esperada | Cifra que devuelve la app | Resultado |
|----------|---------------:|--------------------------:|-----------|
| Días de vacaciones | 16 | | |
| Prima vacacional | **1,115.20** | | |

**Notas:** El test unitario de 836.40 usa 12 días (año 1), no el año 3.
__________________________________________________________________

---

## Caso 10 — Recálculo de enero 2026 con UMA histórica

**Datos de entrada:** salario diario $350.00; 30 días; fecha de pago original
2026-01-31; recálculo ejecutado en noviembre 2026; INFONAVIT inactivo.

**Cálculo manual:** base ISR 10,500 → 420.95 + (10,500 − 7,168.52) × 10.88% =
783.415024. Subsidio de enero 536.21 ⇒ ISR 247.205024. Subsidio de febrero
535.65 ⇒ ISR 247.765024. Diferencia exacta de ISR: 0.56. IMSS obrero enero
265.11.

| Concepto | Cifra esperada | Cifra que devuelve la app | Resultado |
|----------|---------------:|--------------------------:|-----------|
| Percepciones | 10,500.00 | | |
| ISR (enero) | 247.21 | | |
| IMSS obrero | 265.11 | | |
| Deducciones | 512.32 | | |
| Neto | **9,987.68** | | |
| Delta neto vs. subsidio de febrero | **0.56** | | |
| UMA registrada en auditoría | 113.14 | | |
| Subsidio registrado en auditoría | 536.21 | | |

**Notas:** Si se comparan enero y febrero calculados por completo, la diferencia
de neto es 0.95 porque también cambia la UMA. La prueba del proyecto fabrica el
recibo original solo con el ISR de febrero, por eso exige 0.56.

### Variante tope por UMA

Con salario de $10,000/día y pago original 2026-01-31, el SBC tras el recálculo
debe ser 25 × 113.14 = **2,828.50**, no 25 × 117.31 = 2,932.75. La auditoría
debe registrar UMA 113.14 y subsidio 536.21.

---

## Resultado global y firma

| Resumen | Cantidad |
|---------|---------:|
| Casos ✅ | |
| Casos ❌ | |

**Nombre del contador:** _________________________________________

**Despacho:** ____________________________________________________

**Fecha:** ____________________  **Firma:** _______________________

---

## D7 — Revisión con contador de confianza

**Estado: Pendiente de identificar contador para validación.** El desarrollador
debe asignar un contador de confianza para ejecutar los 10 casos y firmar la
validación. Los casos ya están listos con cifras esperadas calculadas a partir
de las fórmulas del proyecto; solo falta la revisión humana.

Cuando se identifique, registrar aquí: nombre del contador, despacho, fecha de
revisión y número de casos conformes. Esta revisión es prerrequisito para el
piloto, junto con la verificación visual de Windows.
