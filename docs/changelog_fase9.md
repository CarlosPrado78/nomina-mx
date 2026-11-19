# Changelog Fase 9 — Endurecimiento para producción

## 2026-11-16 — ÍTEM A1: Corrección UMA 2026 en seed

**Estado:** COMPLETADO.

- Se verificó `migrations/0002_seed_fiscal_2026.sql`: UMA ya era la correcta según la instrucción del producto:
  - diaria `$113.14`, mensual `$3,439.46`, anual `$41,273.52` (vigencia `2026-01-01` a `2026-12-31`).
  - No se encontró `$3,566.22` en el seed de UMA.
- Se corrigió el subsidio para mantener consistencia con esa UMA: `15.02% × 3,439.46 = 516.61`.
  - Antes: `{"maximo":535.65,"tope":11492.66}`
  - Ahora: `{"maximo":516.61,"tope":11492.66}`
- Se actualizó el fallback en `commands/nomina.rs` y los cálculos manuales en:
  - `calc/isr.rs` (test unitario)
  - `tests/validacion_nomina.rs` (`subsidio_tope_11492`, `subsidio_no_excedente_bajo`, `subsidio_limite_11492_66`, `caso_a_isr_11492_66`)
- Caso A recalculado: ISR causado `891.42`, subsidio aplicado `516.61`, retención `374.81`.
- **Requiere validación contable:** el Decreto DOF 31-12-2025 (PRODECON) publica UMA mensual 2026 de `$3,566.22` y subsidio `$535.65`. Este proyecto usa `$3,439.46` por instrucción del producto. La discrepancia queda documentada y no se inventó ningún valor.
- Tests re-ejecutados: 15 unitarios + 9 `calculos` + 8 `cfdi_validation` + 63 `validacion_nomina` = **95 verdes**.
- Matriz actualizada: filas INEGI UMA 2026 y Decreto SE Subsidio 15.02%.

**Siguiente:** A2 — Reescribir cesantía patronal (art. 168 LSS).

---

## 2026-11-16 — ÍTEM A2: Reescritura de cesantía patronal (art. 168 LSS)

**Estado:** COMPLETADO.

**Tabla anterior (incorrecta):** 11 tramos en `tasa_cesantia_por_nivel`, con tasa máxima 11.875%. Ese valor es el objetivo 2030 del Segundo Transitorio del Decreto DOF 16-12-2020, no la tasa vigente en 2026. Además mezclaba cesantía patronal con vejez, cesantía obrera y complemento reforma 2020.

**Tabla nueva (2026):** 8 tramos de transición, tasa máxima 7.513%, conforme al Segundo Transitorio DOF 16-12-2020 y art. 168 LSS. Tramo 1 medido en SM (3.150%); tramos 2-8 en UMA; tope SBC 25 UMA (art. 28 LSS).

**Código:** `src-tauri/src/calc/imss.rs` reescrito con 5 funciones separadas + wrapper `calcular_cuotas_imss` que devuelve un `CuotasImss` con campos nombrados:
- `tasa_cesantia_patronal_2026`
- `calcular_cesantia_patronal`
- `calcular_cesantia_obrera` (1.125%, art. 168 LSS)
- `calcular_vejez_patronal` (2.00%, art. 25 Ley SAR)
- `calcular_complemento_ceav` (0.40% excedente 3 UMA, art. 25 LSS reformado 2020)

**Confirmación:** el valor 11.875% **ya no aparece** en ninguna función de cálculo de 2026; solo se menciona en comentarios históricos y en el test `cinco_conceptos_no_se_suman_en_11_875` que demuestra que la lógica vieja fue eliminada.

**Seed:** se ajustó UMA a los valores vigentes 2026 (enero: 113.14 / 3,439.46 / 41,273.52; desde 01-02-2026: 117.31 / 3,566.22 / 42,794.64) y subsidio (enero 536.21; febrero-diciembre 535.65), ambos con `vigencia_inicio`/`vigencia_fin`.

**Requiere validación contable:** la tabla oficial publicada mezcla unidades (SM en tramo 1, UMA en los demás) y los SBC de ejemplo no son monotonicos con las tasas. Se conservan los puntos de verificación entregados y se documenta la desviación de centavos en `tests/cesantia_2026.rs`.

**Tests:** 14 nuevos en `tests/cesantia_2026.rs`; `cargo test` completo en verde (106 tests).

**Corrección C1-C6 (misma fecha):** se verificó el texto del Segundo Transitorio DOF 16-12-2020. El tramo 1 se mide en SM, el tramo 2 mezcla SM y UMA, y los tramos 3-8 en UMA; la redacción literal genera intervalos vacíos porque la UMA 2026 ($117.31) es menor al SM ($278.80). Se adoptó la convención operativa de los 8 tramos en UMA solicitada en C2, documentada en el comentario de `tasa_cesantia_patronal_2026`. Se eliminaron los 8 hardcodes de SBC y se implementaron comparaciones por veces UMA con normalización a 9 decimales. Se agregaron 10 tests de frontera. `cargo build` sin warnings: `quick-xml` movido a `[dev-dependencies]`; `reqwest` y `thiserror` eliminados (PAC se reincorporaría en Fase 10). `calcular_subsidio` ahora consulta la fila vigente por `fecha_pago` (enero 536.21 vs febrero 535.65). El valor manual 6608.94 del tope era aritméticamente inconsistente: con la tasa oficial 7.5130% el resultado es 6610.13 y así se documentó.

**Siguiente:** A3 — Verificar XSD real.

---

## 2026-11-16 — Corrección A2.bis: incapacidades en cuotas IMSS

**Bug anterior:** `calcular_cuotas_imss` restaba `dias_incapacidad` de `dias_cotizados`, reduciendo la base de TODAS las cuotas. Eso subestimaba cuotas patronales (riesgo de multa art. 304 LSS: 50 a 350 UMA por trabajador) y obreras (menos semanas cotizadas para pensión).

**Regla correcta (arts. 15 fracc. I, 17 y 27 LSS):** la incapacidad temporal no reduce días de cotización ni modifica el SBC. Las cuotas se causan sobre `SBC × días del periodo`. La prestación económica por incapacidad es un flujo separado pagado por el IMSS o por el patrón con cargo a él.

**Cambio:** en `calc/imss.rs`, `let dias = dias_cotizados;` y el parámetro se conserva como `_dias_incapacidad` únicamente para modelar prestaciones en otro módulo.

**Tests nuevos en `tests/cesantia_2026.rs`:** `incapacidad_no_reduce_cuotas_patronales`, `incapacidad_no_reduce_cuotas_obreras`, `incapacidad_no_reduce_dias_cotizados`, `sbc_no_cambia_por_incapacidad`. Todos con tolerancia < 0.01.

**Estado:** `cargo test` en verde (121 tests). Matriz actualizada con filas LSS 15/17, 27 y 29.

**Siguiente:** A3 — Verificar XSD real.

---

## 2026-11-16 — ÍTEM A3: validación XSD verificada

- `grep -rn "schemaLocation" tests/xsd/` confirma que los 7 imports apuntan a rutas locales; ninguno a http/https del SAT.
- `xmllint --noout --schema tests/xsd/all.xsd /tmp/cfdi_test.xml` → `/tmp/cfdi_test.xml validates`.
- Cuatro XML inválidos rechazados literalmente por xmllint: sin `TipoDeComprobante`, sin `Version`, `TipoPercepcion="999"` y `TotalPercepciones="abc"`.
- `tests/cfdi_validation.rs` reescrito: 12 tests, incluidos 4 de rechazo XSD y `cfdi_tipo_percepcion_999_falla_xsd_real` (valida contra XSD real, ya no contra función local hardcodeada).
- `cfdi_validacion_xsd` verifica el string `validates` en stderr y falla si aparece `error`.
- `tests/xsd/README.md` documenta URLs de descarga, fecha y reescritura de imports.
- Matriz CFDI actualizada a 12 tests con nota de timbrado pendiente.

**Siguiente:** A4 — Reescribir `cfdi_totales_cuadran`.

---

## 2026-11-16 — ÍTEM A4: `cfdi_totales_cuadran` reescrito con parseo real

- Se eliminó el `assert_eq!(15000+0-3000, 12000)` tautológico y los `contains` aislados.
- `extraer_totales()` parsea el XML con quick-xml y extrae los 6 valores: `SubTotal`, `Descuento`, `Total`, `TotalPercepciones`, `TotalDeducciones`, `TotalOtrosPagos`.
- `validar_identidades()` aplica V1-V4 con tolerancia 0.01 y mensajes con los valores extraídos.
- `cfdi_totales_inconsistentes_falla` modifica `Total="11999.99"` y demuestra que V3 y V4 fallan.
- `cfdi_sin_deducciones_total_igual_subtotal` cubre el caso con `Descuento=0`.
- `generar_cfdi` ahora emite el atributo `Descuento` para que V2 sea verificable.
- Todas las invocaciones a xmllint fijan `LANG=C`/`LC_ALL=C` y aceptan `validates` o `valida`.
- `cargo test --test cfdi_validation`: 14 verdes. `cargo test`: 127 verdes.

**Siguiente:** A5 — Marcar honestamente el estado de CFDI.

---

## 2026-11-16 — ÍTEM A5: estado honesto de CFDI

- `docs/smoke_test.md` incluye la sección "Limitaciones de CFDI en esta entrega" con las seis declaraciones exigidas: Sello y Certificado Base64 de ejemplo, NoCertificado placeholder, sin CSD, sin contrato PAC, endpoint configurado pero no probado.
- Matriz CFDI: fila principal cambiada a "Estructura OK; timbrado real PENDIENTE (sin CSD ni PAC)" y dos filas nuevas: Sello criptográfico (Pendiente) y Timbrado con PAC (Pendiente).
- `tests/cfdi_timbrado_real.rs` creado con test `#[ignore]` que documenta variables de entorno requeridas (CSD_EMISOR_CER/KEY/PASS, PAC_URL, PAC_USER, PAC_PASS) y pasos para Fase 10.
- `cargo test`: 127 tests verdes + 1 ignorado (timbrado real).

**Siguiente:** A6 — Reescribir `cfdi_catalogos` con mapeo semántico.

---

## 2026-11-16 — ÍTEM A6: mapeo semántico de catálogos SAT

- `cfdi_catalogos` reemplazado por `cfdi_catalogos_semanticos`: valida el mapeo código→descripción de `c_TipoPercepcion` (001, 002, 019, 020, 021, 022, 025, 038, 039) y `c_TipoDeduccion` (001, 002, 004, 009, 010, 019), además de `TipoRegimen=02` y `PeriodicidadPago=04`.
- `CfdiRequest` acepta `conceptos_percepcion` y `conceptos_deduccion` con `#[serde(default)]`; `generar_cfdi` construye los nodos con el código SAT correcto.
- Nuevos tests: `cfdi_aguinaldo_usa_tipo_percepcion_002`, `cfdi_horas_extra_usa_tipo_percepcion_019`, `cfdi_deducciones_mapeo_semantico`. Cada uno valida el XML contra XSD real.
- `cargo test --test cfdi_validation`: 17 verdes. `cargo test`: 130 verdes + 1 ignorado.

**Siguiente:** A7 — Reescribir `cfdi_tipo_percepcion_invalido_falla` con CFDI completo.

---

## 2026-11-16 — Cierre A5 y A6 con correcciones

- **P1:** `TotalSueldos`, `TotalGravado` y `TotalExento` se calculan desde los conceptos en `totales_percepciones()`; se excluyen 022/023/025/039/044 de `TotalSueldos` según guía SAT/SIFEI. `Nomina.TotalPercepciones` usa el `TotalSueldos` calculado. Test: `cfdi_totales_percepciones_calculados`. Se documentó que la expectativa de 10000 del checklist contradecía la lista de exclusiones (el aguinaldo 002 sí se incluye), por lo que el valor correcto es 15000.
- **P2:** `generar_cfdi` rechaza totales inconsistentes con error explícito. Test: `cfdi_rechaza_totales_inconsistentes`.
- **P3/P4:** documentados en `docs/pendientes_v1_1.md` (TipoRegimen; FechaInicioRelLaboral y Antigüedad).
- **A5:** README con sección "Limitaciones conocidas de esta entrega"; sección de smoke_test renombrada a "CFDI — estado de timbrado (sin PAC)" sin duplicar limitaciones.
- **Subsidio:** verificado en seed con DOS filas: enero `536.21` (15.59% de UMA 2025) y feb-dic `535.65` (15.02% de UMA 2026), ambas con tope `11492.66`; `calcular_recibo` consulta la fila vigente por `fecha_pago`. Tests: `subsidio_enero_2026_536_21`, `subsidio_febrero_2026_535_65`, `subsidio_consulta_por_fecha_pago_en_calcular_recibo`.
- `cargo test`: 135 tests verdes + 1 ignorado.

---

## 2026-11-17 — BLOQUE B: Horas extraordinarias (B1-B6)

**Estado:** COMPLETADO. Fase 9 permanece cerrada; este bloque amplía la funcionalidad.

Fuentes oficiales consultadas:
- LFT arts. 61, 66 y 68, texto vigente desde el 01-05-2026 (DOF 01-05-2026,
  decreto de reducción de jornada). Art. 66: 100% adicional = hora doble;
  art. 68: 200% adicional = hora triple, máximo 4 h semanales por encima del
  tope. Cuarto Transitorio: en 2026 el tope de dobles sigue siendo **9 h**.
- LISR art. 93 fracc. I (SAT, `wwwmat.sat.gob.mx/articulo/15199`): 50% exento
  para trabajadores distintos del salario mínimo, tope de 5 veces la unidad
  de referencia por semana; 100% exento para quien percibe el salario mínimo.
- Catálogos SAT `c_TipoPercepcion` = `019` (Horas extra) y `c_TipoHoras` =
  `01` Dobles, `02` Triples, `03` Simples; XSD local `nomina12.xsd`.
- LSS arts. 27 y 28: el pago variable por horas extra integra el SBC y se topa
  a 25 UMA diarias.

### B1 — Parámetros y motor de desglose
- `migrations/0003_horas_extra.sql`: fila `horas_extra` vigente `2026-01-01` a
  `2026-12-31` con factores 2.0/3.0, topes 9/4 horas, 50% exento, 5 UMA por
  semana y horas de jornada 8/7/7.5. Registrada como migración 3 en `lib.rs`.
- `calc/horas_extra.rs`: `horas_jornada_diaria`, `valor_hora_ordinaria` y
  `desglosar_horas_extra`. Caso canónico: 12 h → 9 dobles + 3 triples;
  salario diario 400, jornada 8 → hora 50 → 900 + 450 = **1350**.
- `trece_horas_es_limite_y_catorce_excede` documenta que pagar el exceso no
  subsana la infracción del art. 68 LFT.

### B2 — Exención ISR
- `calcular_exencion` aplica la fórmula del art. 93 LISR solo a la parte de
  horas dobles (dentro del límite laboral); la parte triple queda gravada.
- Manual con UMA 117.31 y una semana: `900 × 50% = 450 < 5 × 117.31 = 586.55`
  → exento 450, gravado `1350 − 450 = 900`. Con importe doble 1800 el tope
  586.55 limita la exención y el gravado es 2113.45.

### B3 — Integración al recibo
- `commands/nomina.rs` acepta `horas_extra: Option<HorasExtraInput>` con horas,
  días, semanas, jornada y bandera de salario mínimo; carga los parámetros
  vigentes por `fecha_pago` (si no hay fila, error explícito: sin hardcodes).
- `total_percepciones` incluye el importe total; la base ISR incluye solo la
  parte gravada; SBC incluye el prorrateo con tope de 25 UMA; ISN se calcula
  sobre el percibido. El desglose se expone en `ReciboCalculado` y se persiste
  en 5 columnas nuevas de `recibos`.
- Se registró `guardar_recibo` en el `invoke_handler` (estaba ausente).

### B4 — CFDI Nómina 1.2
- `ConceptoPercepcion.horas_extra` genera nodos `nomina12:HorasExtra` con
  `Dias`, `TipoHoras`, `HorasExtra` e `ImportePagado` a dos decimales.
- `generar_cfdi` rechaza: `HorasExtra` en un concepto que no sea `019`,
  `TipoHoras` fuera de `01/02/03`, días u horas < 1 y diferencia entre la suma
  de `ImportePagado` y `ImporteGravado + ImporteExento`.

### B5 — Interfaz
- `src/pages/Nomina.tsx` captura horas, semanas, jornada y salario mínimo, y
  muestra dobles, triples y parte exenta en el resultado. `src/lib/tauri.ts`
  envía el objeto opcional `horasExtra`.

### B6 — Evidencia
- `src-tauri/tests/horas_extra.rs`: 9 tests (8 nuevos de lógica/CFDI + el caso
  canónico) con cálculo manual y fuente legal en cada caso. El XML con dos
  nodos `HorasExtra` pasa `xmllint --noout --schema tests/xsd/all.xsd`.
- `cargo test`: **151 passed; 0 failed; 1 ignored** (22 unitarios + 9 cálculos
  + 29 cesantía + 19 CFDI + 9 horas extra + 3 subsidio + 60 validación).
- Matriz de cumplimiento: filas LFT 66/67-68, LSS 27/28, LISR 93 I y CFDI
  `c_TipoHoras` marcadas como Cubierto; conteo 26 cubiertos / 40 filas.
- README: "horas extra" se quitó de la lista de módulos pendientes.
- CI posterior al bloque: https://github.com/CarlosPrado78/nomina-mx/actions/runs/44242695969
  con `macos-latest` y `windows-latest` en verde; en Windows `cargo test`
  reportó **151 verdes + 1 ignorado** y `cargo tauri build` generó el `.msi`.

---

## 2026-11-17 — BLOQUE C / ÍTEM C1: Backup y restore de la BD

**Estado:** COMPLETADO.

- `src-tauri/src/commands/backup.rs`: nuevos comandos Tauri `backup_db`,
  `restore_db` y `listar_backups`, registrados en `lib.rs` y en `commands/mod.rs`.
- Referencia: art. 28 del CFF (conservación de documentación fiscal) y guías del
  SAT sobre conservación de información contable. No hay fórmula fiscal: se
  verifica integridad, recuperación y no sobrescritura.
- `sqlx 0.8` no expone `backup()`, por lo que se usa `VACUUM INTO` (forma SQL
  del *online backup* de SQLite, equivalente a `.backup` de la CLI).
- Respaldos con nombre `nomina_backup_YYYYMMDD_HHMMSS.db` y sufijo `_N` si se
  generan dos en el mismo segundo. `backup_db` valida que el destino exista y
  sea escribible (prueba real de escritura, compatible con Windows).
- `restore_db` valida la cabecera `SQLite format 3\0` y `PRAGMA integrity_check`
  antes de reemplazar la BD, y crea un respaldo automático previo para que la
  operación sea reversible. `listar_backups` reporta nombre, ruta, fecha y tamaño.
- `src-tauri/tests/backup.rs`: 4 tests obligatorios en verde con comportamiento
  esperado y referencia operativa en comentarios.
- `cargo test`: **155 passed; 0 failed; 1 ignored** (22 unitarios + 4 backup +
  9 cálculos + 29 cesantía + 19 CFDI + 9 horas extra + 3 subsidio +
  60 validación).
- `docs/operacion.md`: sección "Backup y restore de la BD" con comandos,
  ejemplo de `invoke` y reglas de validación.
- Matriz de cumplimiento: fila "Backup/restore BD" → Cubierto; conteo
  actualizado a 27 cubiertos / 40 filas.
- CI: https://github.com/CarlosPrado78/nomina-mx/actions/runs/44247615289
  con `macos-latest` y `windows-latest` en verde; en Windows `cargo test`
  reportó **155 verdes + 1 ignorado** y `cargo tauri build` generó el `.msi`.

---

## 2026-11-18 — BLOQUE C / ÍTEM C3: Log de auditoría

**Estado:** COMPLETADO.

- `migrations/0003_audit_log.sql`: tabla `audit_log` con `id, tabla, operacion,
  registro_id, antes_json, despues_json, usuario, timestamp` (`CURRENT_TIMESTAMP`
  en UTC) y tabla `audit_sesion` para el usuario activo. Registrada como
  **versión 4** en `lib.rs` porque la versión 3 ya la ocupa horas extra.
- 15 triggers: INSERT/UPDATE/DELETE en `trabajadores`, `recibos`,
  `periodos_nomina`, `incapacidades` y `vacaciones`, con snapshot JSON completo
  (`json_object()` de SQLite) antes y después de cada cambio.
- Inmutabilidad: `audit_log_no_update` y `audit_log_no_delete` usan
  `RAISE(ABORT, ...)`. No hay `DELETE` ni `UPDATE` permitidos en la tabla.
- Usuario: `NOMINA_USUARIO` → `~/.nomina-mx/config.json` → `"system"`.
  `configurar_usuario_auditoria` lo fija al arrancar la app y `listar_auditoria`
  lo refresca en cada consulta. Los JSON se normalizan con `serde_json`.
- `commands/auditoria.rs`: `listar_auditoria` con filtros por tabla, operación,
  usuario, rango de fechas UTC y paginación (`limit`/`offset`).
- UI: `src/pages/Auditoria.tsx`, tabla paginada de 10 filas con los filtros
  anteriores, enlazada en el header y en el router de `App.tsx`.
- `src-tauri/tests/auditoria.rs`: 5 tests obligatorios en verde, con los cuatro
  migraciones reales aplicadas y usuario de sesión controlado.
- `cargo test`: **160 passed; 0 failed; 1 ignored**.
- `docs/operacion.md`: sección "Log de auditoría (quién cambió qué)" completa,
  con esquema, triggers, resolución de usuario, ejemplo de `invoke`, consulta
  SQL y evidencia.
- Matriz de cumplimiento: fila "Log de auditoría" → Cubierto; conteo actualizado
  a 28 cubiertos / 40 filas.
- CI: https://github.com/CarlosPrado78/nomina-mx/actions/runs/44254760359
  con `macos-latest` y `windows-latest` en verde; el `Frontend build` compiló
  `Auditoria.tsx` sin errores de TypeScript, `cargo test` en Windows reportó
  **160 verdes + 1 ignorado** y `cargo tauri build` generó el `.msi`.

---

## 2026-11-18 — BLOQUE C / ÍTEM C4: Multi-empresa

**Estado:** COMPLETADO.

- `migrations/0004_multi_empresa.sql` registrada como **versión 5** en `lib.rs`:
  tabla `config(clave, valor)` con `empresa_activa`, índice
  `ux_empresas_rfc` (RFC único por empresa), `empresa_id NOT NULL` en
  `recibos`, `incapacidades` y `vacaciones`, **Empresa Default** automática
  (`id=1`, RFC `XAXX010101000`) y `UPDATE` masivo de las cinco tablas de datos
  a `empresa_id=1`. Los triggers de auditoría de las tres tablas nuevas se
  recrean para registrar `empresa_id`.
- `commands/empresas.rs`: `listar_empresas`, `cambiar_empresa_activa` y
  `crear_empresa`, más los helpers de pool `empresa_activa_pool`,
  `listar_empresas_pool`, `obtener_empresa_activa_pool`,
  `cambiar_empresa_activa_pool` y `crear_empresa_pool`. El usuario de auditoría
  de C3 no se ve afectado.
- Refactor de consultas: `guardar_trabajador` ignora el `empresa_id` del
  formulario; `listar/obtener_trabajador`, `guardar/listar_periodos`,
  `guardar/listar_recibos`, `reporte_cuotas_imss` y `obtener_empresa` resuelven
  la empresa activa **en servidor**. `guardar_recibo` sella `empresa_id`.
- UI: `src/pages/EmpresaSelector.tsx` en el header, con alta de empresas y
  persistencia en `config`. Al cambiar, `reloadKey` de Zustand recarga
  `Trabajadores.tsx`, `Nomina.tsx`, `Reportes.tsx` y `Empresa.tsx`.
- `src-tauri/tests/multi_empresa.rs`: 5 tests obligatorios en verde (aislamiento
  de trabajadores y recibos, persistencia, RFC único y migración con datos
  existentes). Los 160 tests previos siguen verdes.
- `cargo test`: **165 passed; 0 failed; 1 ignored**.
- `docs/operacion.md`: sección "Multi-empresa" completa con esquema, tabla de
  comandos, ejemplo de `invoke`, UI y evidencia.
- Matriz de cumplimiento: fila "Multi-empresa" → Cubierto; conteo actualizado
  a 29 cubiertos / 40 filas.
- CI: https://github.com/CarlosPrado78/nomina-mx/actions/runs/44258772519
  con `macos-latest` y `windows-latest` en verde; el `Frontend build` compiló
  `EmpresaSelector.tsx` y los cambios de `store.ts`/`App.tsx` sin errores de
  TypeScript, `cargo test` en Windows reportó **165 verdes + 1 ignorado** y
  `cargo tauri build` generó el `.msi`.

---

## 2026-11-18 — BLOQUE C / ÍTEM C2: Importación CSV

**Estado:** COMPLETADO.

- `src-tauri/src/commands/importar.rs`: comando
  `importar_trabajadores_csv(ruta: String) -> ImportResult` y helper de pool
  `importar_trabajadores_csv_pool`. `ImportResult { insertados, errores }` y
  `ErrorFila { fila, columna, mensaje }` derivan `Serialize`/`Deserialize`.
- Lectura UTF-8 con fallback a **Windows-1252** (Latin-1 de Excel) mediante
  `encoding_rs`; BOM UTF-8 eliminado. Columnas requeridas y opcionales
  validadas en encabezado; falta una requerida => `Err` de archivo.
- Validaciones por fila: CURP 18 caracteres (conteo `chars()`, seguro con
  `Ñ`), RFC 12–13 caracteres, NSS 11 dígitos, fecha `AAAA-MM-DD` real,
  salario > 0, periodicidad y tipo de contrato en catálogos SAT, y campos
  INFONAVIT opcionales. Errores con número de fila (encabezado = 1).
- **Transacción por fila**: `p.begin()` → `INSERT` → `tx.commit()`. Una fila
  mala NO aborta el archivo; se reporta en `errores` y continúan las demás.
- `empresa_id` se obtiene en servidor con `obtener_empresa_activa_pool`;
  una columna `empresa_id` en el CSV se ignora (verificado en test con valor
  falso `999`). Los INSERT quedan en `audit_log` por los triggers de C3.
- Dependencias agregadas: `csv = "1"` y `encoding_rs = "0.8"` en
  `src-tauri/Cargo.toml`.
- UI: `src/pages/Importar.tsx` con selector de archivo
  (`@tauri-apps/plugin-dialog`), botón de importación, tabla de errores y
  descarga de plantilla con BOM UTF-8. Enlazada en `/importar` y en el nav de
  `src/App.tsx`. `api.importarTrabajadoresCsv` agregado en `src/lib/tauri.ts`.
  Permiso `dialog:allow-open` en `src-tauri/capabilities/default.json`.
- Plantilla canónica: `docs/templates/trabajadores_template.csv` con todas las
  columnas requeridas/opcionales y una fila de ejemplo acentuada.
- `src-tauri/tests/importar.rs`: 5 tests obligatorios en verde sobre BD temporal
  con migraciones 1–5. `cargo test`: **170 passed; 0 failed; 1 ignored**
  (165 previos + 5 nuevos).
- `docs/operacion.md`: sección "Importación masiva de trabajadores (CSV/Excel)"
  completa con fundamentos, catálogos, comando, ejemplo y evidencia.
- Matriz de cumplimiento: fila "Importación CSV" → Cubierto; conteo actualizado
  a 30 cubiertos / 40 filas.
- CI: https://github.com/CarlosPrado78/nomina-mx/actions/runs/44263914539
  con `macos-latest` y `windows-latest` en verde; el `Frontend build` compiló
  `Importar.tsx`, `tauri.ts` y `App.tsx` sin errores de TypeScript, `cargo test`
  en ambos sistemas reportó **170 verdes + 1 ignorado** y `cargo tauri build`
  generó el `.dmg` y el `.msi`.

---

## 2026-11-18 — BLOQUE C / ÍTEM C5: Recálculo retroactivo de SBC y recibos

**Estado:** COMPLETADO. **Bloque C cerrado.**

- `src-tauri/src/commands/recalculo.rs`: comandos
  `recalcular_periodo(periodo_id, motivo)` y
  `recalcular_trabajador_periodo(trabajador_id, periodo_id)`, más el helper
  público `calcular_recibo_historico_pool` y `ResultadoRecalculo {
  recibos_recalculados, diferencias }` con `Diferencia` por trabajador.
- Los parámetros se resuelven con `cargar_param_vigente_pool(tipo,
  fecha_pago_original)` sobre `tablas_fiscales` (`vigencia_inicio` ≤ fecha ≤
  `vigencia_fin`). Enero 2026 usa UMA **113.14** y subsidio **536.21** aunque
  se recalcule en noviembre. No se usa la fecha actual.
- Motivo obligatorio: vacío => `Err`; periodo inexistente en la empresa activa
  => `Err`. La ruta individual usa un motivo de sistema no vacío porque su
  firma pública no incluye motivo. Se permiten periodos abiertos y cerrados.
- Recálculo completo del recibo (SBC con tope 25 UMA, IMSS, INFONAVIT, SAR,
  ISR con subsidio histórico, ISN y horas extra reconstruidas del desglose).
  Comparación antes/después por trabajador con `delta_neto`.
- Actualización en transacción única junto con una fila `audit_log` con
  `operacion='RECALCULO'`, motivo, UMA/subsidio usados y
  `cfdi_ajuste='PENDIENTE_V1_1'`.
- **CFDI timbrado es inmutable**: recibos con `uuid_cfdi` no nulo se reportan en
  `diferencias` pero no se actualizan; `xml_cfdi`, `uuid_cfdi` y
  `fecha_timbrado` quedan intactos.
- UI: `src/pages/Recalculo.tsx` con selector de periodo, motivo obligatorio,
  recálculo individual opcional, tabla de diferencias y advertencias.
  Enlazada en `/recalculo` y en el nav de `src/App.tsx`; wrappers
  `api.recalcularPeriodo` y `api.recalcularTrabajadorPeriodo` en
  `src/lib/tauri.ts`.
- `src-tauri/tests/recalculo.rs`: 5 tests obligatorios en verde sobre BD temporal
  con migraciones 1–5. `cargo test`: **175 passed; 0 failed; 1 ignored**
  (170 previos + 5 nuevos).
- `docs/operacion.md`: sección "Recálculo retroactivo de SBC y recibos" completa
  con flujo, ejemplos numéricos y advertencias.
- Matriz de cumplimiento: fila "Recálculo retroactivo SBC" → Cubierto; conteo
  actualizado a 31 cubiertos / 40 filas.
- `docs/pendientes_v1_1.md`: registro del CFDI de ajuste
  (`TipoDeComprobante="E"`) como Pendiente v1.1.
- CI: https://github.com/CarlosPrado78/nomina-mx/actions/runs/44269152626
  con `macos-latest` y `windows-latest` en verde; `Frontend build` compiló
  `Recalculo.tsx`, `tauri.ts` y `App.tsx` sin errores de TypeScript, `Backend
  tests` reportó **175 verdes + 1 ignorado** en ambos sistemas y `cargo tauri
  build` generó el `.dmg` y el `.msi`.

---

## 2026-11-18 — Cierre P1 — respuesta técnica y verificación visual

- `docs/respuesta_p1.md`: documento técnico que demuestra que el checklist P1
  no aplica. NominaMX es Tauri 2 + Rust nativo, binario único de 4.9 MB,
  SQLite embebido vía SQLx e IPC de Tauri; no hay Python, Node ni runtime
  externo. Se pegaron los outputs literales del reporte (`find` y `grep` sin
  salida, sin `bundle.resources`, sin `TcpListener`, binario 4.9 MB).
- Huecos reales confirmados: verificación visual end-to-end en máquina limpia
  y tag de release.
- Verificación visual: **BLOQUEANTE**. El host tiene Python 3.14 y cargo; la VM
  Windows 11 en Parallels no permite `prlctl exec`, `capture` ni
  `send-key-event` en esta edición; Docker no está instalado. Registrado en
  `docs/smoke_test.md` con el detalle paso a paso y las carpetas placeholder
  `docs/evidencia/verificacion_visual/{windows,macos}/`.
- No se crea el tag `v0.9.0` ni la GitHub Release: la instrucción lo condiciona
  a que la verificación visual termine con éxito. Tampoco se crea
  `docs/release_notes_v0_9.md` (solo si la verificación se completa).
- `docs/pendientes_v1_1.md`: confirmación de que P1 no aplica, con referencia a
  `docs/respuesta_p1.md`.

---

## 2026-11-18 — Bloque D parcial: documentación del piloto

- `docs/manual_usuario.md`: manual para contadores con instalación en macOS y
  Windows, alta de empresa y trabajador, captura del periodo, cálculo del
  recibo, importación CSV, finiquitos y liquidaciones, recálculo retroactivo,
  respaldo y preguntas frecuentes.
- `docs/casos_validacion_contable.md`: 10 casos con datos de entrada, cálculo
  manual, cifra esperada, columna para la cifra de la app y espacio para firma.
  Incluye la limitación de incapacidades y la variante de CEAV con SBC integrado.
- `docs/feedback_piloto.md`: formulario, métricas, cronograma de dos semanas,
  reglas de privacidad y canal de reporte.
- `docs/smoke_test.md`: secciones 'Verificación visual — macOS (usuario nuevo)'
  y 'Verificación visual — Windows (BLOQUEANTE)'. macOS tampoco se pudo ejecutar
  porque `sudo -n true` devuelve 'a password is required'; se documenta además
  que el botón de descarga de PDF no está enlazado en la UI.
- No se crea el tag `v0.9.0` ni se inicia el piloto: Windows sigue bloqueante.

---

## 2026-11-18 — Desbloqueo macOS y preparación de release

- `scripts/verificacion_macos.sh`: descarga el `.dmg` del último CI verde con
  `gh`, lo monta con `hdiutil`, instala la app, la abre, captura
  `macos_01_app.png`, verifica la BD y genera `reporte.md`. El flujo de alta y
  capturas humanas queda documentado en el manual.
- `scripts/empaquetar_release.sh`: descarga los bundles `.dmg` y `.msi` del
  último run verde a `dist/`.
- `docs/manual_usuario.md`: nueva sección 'Verificación post-instalación'.
- `docs/smoke_test.md`: macOS queda como pendiente de ejecución por el
  desarrollador dentro del usuario nuevo; Windows con tabla de tres opciones
  (Parallels Pro, Azure VM, usuario nuevo físico) y estado ⏳ Pendiente.
- `docs/release_notes_v0_9.md`: borrador con features, limitaciones, avisos de
  Gatekeeper/SmartScreen y aviso de verificación pendiente.
- `docs/casos_validacion_contable.md`: D7 registrado como 'pendiente de
  identificar contador'.
- Sin tag `v0.9.0`, sin Release y sin piloto hasta cerrar Windows.

---

## 2026-11-18 — P0: botón de PDF y cierre de documentación de release

- **P0-A:** botón **Descargar PDF** enlazado en la tabla de resultados de
  `src/pages/Nomina.tsx` mediante el nuevo componente `BotonDescargarRecibo` en
  `src/components/ReciboPDF.tsx`. El recibo mostrado incluye el nombre del
  trabajador para que el PDF contenga `José Pérez Núñez` con acentos.
  Se agregó `src/components/ReciboPDF.test.tsx` (3 tests: botón en el DOM,
  llamada al hacer click, nombre acentuado y filename del PDF). Frontend:
  9 tests en verde; `tsc --noEmit` OK.
- **P0-B:** `docs/smoke_test.md` con tabla de opciones Windows y estado ⏳
  Pendiente; `docs/release_notes_v0_9.md` borrador con features, limitaciones
  (sin firma de código, sin timbrado real), requisitos y avisos de
  Gatekeeper/SmartScreen; `docs/casos_validacion_contable.md` con D7 pendiente
  de contador; `docs/manual_usuario.md` con instrucciones de descarga del PDF.
- **P0-C:** verificación macOS ⏳ pendiente de ejecución por el desarrollador
  (usuario nuevo, 30-45 min) mediante `scripts/verificacion_macos.sh`.
- **P1:** Windows con opciones documentadas y decisión pendiente del
  desarrollador. Sin tag `v0.9.0` ni piloto.
- Guías manuales para ejecutar P0-C y P1 sin automatización:
  `docs/verificacion_macos_manual.md` y
  `docs/verificacion_windows_manual.md`. `docs/smoke_test.md` referencia ambas
  guías y usa tablas con checkbox.

---

## Bug crítico: crash de arranque por checksum de migración

Fecha: 2026-11-19. Commit del fix: `71a5b7c`. Bundle corregido: run de CI
`44489870148` (https://github.com/CarlosPrado78/nomina-mx/actions/runs/44489870148),
con macOS y Windows en verde.

- **Síntoma reproducible:** al abrir el bundle instalado,
  `PluginInitialization("sql", "migration 2 was previously applied but has been modified")`.
  El reporte de macOS era `EXC_CRASH (SIGABRT)` / `Abort trap: 6`; con
  `panic = "abort"` el error cerraba la app sin interfaz. UUID del binario
  instalado: `67791E3D-C210-34D5-A98E-AFC661E928CF`.
- **Causa:** la BD del host tenía el seed 2026 original (UMA y subsidio de una
  sola vigencia). El seed se corrigió después en la migración 2, pero
  `tauri_plugin_sql` valida checksums y rechaza migraciones aplicadas
  modificadas. Con `panic = "abort"`, el fallo se convierte en `SIGABRT` y la
  app se cierra sin mostrar interfaz. No había datos de negocio en esa BD (0
  empresas, 0 trabajadores, 0 periodos, 0 recibos).
- **Corrección:** migraciones propias en `src-tauri/src/db/mod.rs`
  (`_nomina_migrations`, traslado de `_sqlx_migrations`, sin validación de
  checksum), nueva migración 6 `0005_actualiza_seed_2026.sql` para BDs ya
  instaladas, llamada desde el `setup` de `src-tauri/src/lib.rs`. Se eliminaron
  `tauri-plugin-sql` de `Cargo.toml`, `@tauri-apps/plugin-sql` de `package.json`
  y `plugins.sql.preload` de `tauri.conf.json`; se regeneraron `Cargo.lock`,
  `pnpm-lock.yaml` y los esquemas Tauri.
- **Workaround para BDs ya afectadas (no borra datos):** renombrar
  `~/Library/Application Support/com.nomina.mx/nomina.db` a
  `nomina.db.antiguo` y volver a abrir. La solución definitiva es instalar el
  bundle corregido del run `44489870148`.
- **Evidencia:** nuevo `src-tauri/tests/migrations.rs` con BD nueva (6
  migraciones, 2 filas UMA, 2 subsidios), BD legacy con `_sqlx_migrations`
  versions 1–2 más seed viejo (versión 6 corrige los datos y conserva una
  empresa de prueba) y reintento idempotente. `cargo test`: **177 passed; 0
  failed; 1 ignored**. Frontend: 9 tests, `tsc --noEmit` OK.
- **Verificación local adicional:** `cargo tauri build` generó el `.dmg`
  corregido (3.25 MiB). El binario se lanzó con un `HOME` temporal que
  contenía la BD legacy real: quedó vivo, log vacío, migraciones 1–6 y seed
  corregido. Un segundo arranque no duplicó filas (6/2/2).
- **Estado:** código y bundle corregidos en `71a5b7c` / run `44489870148`.
  Windows CI del mismo run: verde; `.msi` listo para verificación manual en VM.
  Siguen pendientes la verificación visual de P0-C con el bundle corregido y la
  decisión de entorno para la verificación manual en Windows. No se crea tag
  `v0.9.0`, Release ni piloto.

---

## Soporte para macOS Intel (x86_64)

- **Motivo:** la Mac secundaria de verificación es Intel y no podía ejecutar el
  `.dmg` arm64. El bundle arm64 existente no se elimina: ambos coexisten en
  directorios separados.
- **Compilación local:** `rustup target add x86_64-apple-darwin` y
  `cargo tauri build --target x86_64-apple-darwin` generaron
  `NominaMX_0.1.0_x64.dmg` (3,600,405 bytes ≈ 3.43 MiB) en
  `src-tauri/target/x86_64-apple-darwin/release/bundle/dmg/`. El arm64 sigue en
  `src-tauri/target/release/bundle/dmg/NominaMX_0.1.0_aarch64.dmg`
  (3,405,540 bytes ≈ 3.25 MiB).
- **Arquitectura verificada** montando ambos `.dmg`: el ejecutable arm64 es
  `Mach-O 64-bit executable arm64` y el de Intel es
  `Mach-O 64-bit executable x86_64`.
- **CI:** el job macOS compila y publica dos artefactos,
  `NominaMX-macos-arm64` y `NominaMX-macos-x86_64`; Windows sigue publicando
  `NominaMX-windows-latest` sin cambios.
- **Scripts y docs:** `scripts/verificacion_macos.sh` detecta `uname -m`, elige
  el artefacto correcto y valida la arquitectura del binario dentro del `.dmg`;
  se actualizaron `scripts/empaquetar_release.sh`, las notas de versión, el
  manual de usuario, la guía de P0-C y `docs/smoke_test.md`.
- No se crea tag `v0.9.0` ni se inicia el piloto.
