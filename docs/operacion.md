# Operación

## Importación masiva de trabajadores (CSV/Excel)

Estado: **Cubierto** (ÍTEM C2). Implementado en
`src-tauri/src/commands/importar.rs`, registrado en el `invoke_handler` de
`src-tauri/src/lib.rs` y enlazado en la UI `src/pages/Importar.tsx` (ruta
`/importar`). El alta individual sigue disponible en
`src/pages/Trabajadores.tsx` vía `guardar_trabajador`.

### Fundamento y reglas

- Referencias: CFF art. 28 (conservación de registros), LFT arts. 2 y 25
  (relación de trabajo y salario) y catálogos del SAT `c_PeriodicidadPago` y
  `c_TipoContrato` usados por el CFDI de Nómina 1.2.
- El archivo se lee en el backend. Se intenta **UTF-8** primero y, si los bytes
  no son válidos, se decodifica **Windows-1252** (superconjunto de Latin-1
  habitual en Excel en español). Se elimina un BOM UTF-8 inicial si existe.
- El encabezado es la **fila 1**; la primera fila de datos es la **fila 2**.
  Los errores reportan el número de fila real del CSV.
- Si falta una columna requerida, el comando devuelve `Err` de archivo y no
  inserta nada. Si una fila es inválida, se acumula en `ImportResult.errores`
  y **las demás filas se procesan igual**.
- Cada fila válida se inserta en su **propia transacción** (`p.begin()` →
  `tx.commit()`). No hay transacción global por archivo.
- `empresa_id` **nunca se lee del CSV**: se resuelve en servidor con
  `obtener_empresa_activa_pool`. Una columna `empresa_id` en el archivo se
  ignora. Todas las altas quedan registradas en `audit_log` por los triggers
  de C3.

### Columnas

Requeridas: `nombre`, `curp`, `rfc`, `nss`, `fecha_ingreso`,
`salario_diario`, `periodicidad_pago`, `tipo_contrato`.

Opcionales: `num_empleado`, `departamento`, `puesto`, `infonavit_activo`,
`infonavit_factor`, `infonavit_tipo` (por defecto `CUOTA_FIJA_VSM`).

| Campo | Validación |
|-------|------------|
| `curp` | 18 caracteres con formato CURP (expresión regular del SAT) |
| `rfc` | 12 o 13 caracteres; acepta `Ñ` y `&` |
| `nss` | Exactamente 11 dígitos ASCII |
| `fecha_ingreso` | `AAAA-MM-DD` y fecha calendario válida |
| `salario_diario` | Numérico finito > 0 |
| `periodicidad_pago` | `diaria`, `semanal`, `quincenal`, `mensual`, `bimestral`, `trimestral`, `semestral`, `anual` |
| `tipo_contrato` | `01`/`indefinido`, `02`/`temporal`, `03`/`obra_determinada` (y alias en español) |
| `infonavit_activo` | `1`/`0`, `true`/`false`, `si`/`no` |
| `infonavit_factor` | Numérico finito ≥ 0 |

La plantilla canónica está en `docs/templates/trabajadores_template.csv` y se
puede descargar desde la propia pantalla (BOM UTF-8 para Excel).

### Comando Tauri

```ts
interface ErrorFila {
  fila: number;       // número de fila del CSV (encabezado = 1)
  columna: string;    // columna inválida, o "fila"/"base_de_datos"
  mensaje: string;
}

interface ImportResult {
  insertados: number;
  errores: ErrorFila[];
}

const resultado = await invoke<ImportResult>('importar_trabajadores_csv', {
  ruta: '/ruta/absoluta/trabajadores.csv',
});
console.log(resultado.insertados, resultado.errores);
```

También está expuesto como `api.importarTrabajadoresCsv(ruta)` en
`src/lib/tauri.ts`. La UI usa `@tauri-apps/plugin-dialog` (permiso
`dialog:allow-open` en `src-tauri/capabilities/default.json`) para elegir el
archivo y muestra una tabla con fila, columna y mensaje de cada error.

### Evidencia

`src-tauri/tests/importar.rs` aplica las migraciones 1–5 sobre SQLite temporal
y cubre los cinco escenarios obligatorios:

1. `importar_csv_valido_inserta_todos` — 5 filas → 5 insertados, 0 errores,
   `empresa_id=1` aunque el CSV intente enviar `999`, y 5 registros en
   `audit_log`.
2. `importar_csv_con_una_fila_invalida_procesa_el_resto` — 5 filas, CURP mala
   en la fila CSV 3 → 4 insertados y 1 error en `curp`.
3. `importar_csv_falta_columna_requerida_falla` — sin `curp` → `Err`, 0 altas.
4. `importar_csv_rfc_invalido_no_inserta` — RFC malo → 0 altas y error en
   columna `rfc`, fila 2.
5. `importar_csv_encoding_latin1_funciona` — `José Ángel` en Windows-1252 se
   conserva con acentos en BD.

## Backup y restore de la BD

Estado: **Cubierto** (ÍTEM C1). Implementado en `src-tauri/src/commands/backup.rs`
y registrado en el `invoke_handler` de `src-tauri/src/lib.rs`.

La BD activa se encuentra en:

- macOS: `~/Library/Application Support/com.nomina.mx/nomina.db`
- Windows: `%APPDATA%\com.nomina.mx\nomina.db`

`sqlx 0.8` no expone la API `.backup()` de SQLite, así que se usa la sentencia
SQL `VACUUM INTO`, que es la forma SQL del mecanismo *online backup* de SQLite
(equivalente a `.backup` de la CLI). Genera un archivo consistente sin bloquear
la BD activa. Referencia operativa: art. 28 del CFF y guías del SAT sobre
conservación de información contable.

### Comandos Tauri

| Comando | Parámetros | Devuelve |
|---------|------------|----------|
| `backup_db` | `destino: String` (directorio; vacío = `<app_data_dir>/backups`) | Ruta del respaldo creado |
| `restore_db` | `origen: String` (archivo `.db` validado) | Ruta del respaldo automático previo |
| `listar_backups` | ninguno | `Vec<ResumenBackup>` con `nombre`, `ruta`, `fecha` y `tamano` |

### Ejemplo de uso desde el frontend

```ts
import { invoke } from '@tauri-apps/api/core';

interface ResumenBackup {
  nombre: string;
  ruta: string;
  fecha: string;   // YYYYMMDD_HHMMSS[_N]
  tamano: number;  // bytes
}

// 1. Crear un respaldo en un directorio que ya existe y sea escribible.
const ruta = await invoke<string>('backup_db', {
  destino: '/Users/usuario/Documents/respaldos',
});

// 2. Listar respaldos disponibles.
const respaldos = await invoke<ResumenBackup[]>('listar_backups');

// 3. Restaurar: valida cabecera + integridad y guarda primero el estado actual.
const respaldoPrevio = await invoke<string>('restore_db', {
  origen: respaldos[0].ruta,
});
```

### Reglas verificadas

- El nombre es `nomina_backup_YYYYMMDD_HHMMSS.db`. Si ya existe (dos backups en
  el mismo segundo), se agrega el sufijo `_1`, `_2`, ...: nunca se sobrescribe.
- `backup_db` valida que el directorio destino exista y sea escribible antes de
  escribir; en Windows la prueba se hace creando y borrando un archivo temporal.
- `restore_db` rechaza el archivo si no empieza con `SQLite format 3\0` o si
  `PRAGMA integrity_check` no devuelve `ok`. La BD activa no se toca si la
  validación falla.
- Antes de reemplazar la BD, `restore_db` crea automáticamente un respaldo del
  estado actual en `<app_data_dir>/backups`, de modo que la operación es
  reversible.
- Evidencia: `src-tauri/tests/backup.rs` con
  `backup_genera_archivo_valido`, `restore_reemplaza_bd`,
  `restore_rechaza_archivo_corrupto` y `backup_no_sobrescribe`.

## Log de auditoría (quién cambió qué)

Estado: **Cubierto** (ÍTEM C3). Implementado en
`src-tauri/src/commands/auditoria.rs` y `migrations/0003_audit_log.sql`.
Referencias: CFF art. 28 (conservación de información y registros) y LSS art. 15
(registro de movimientos afiliatorios).

> Nota de versión: el archivo se llama `0003_audit_log.sql` por la definición
> del índice, pero se registra como migración **4** en `lib.rs` porque la
> versión 3 ya está ocupada por `0003_horas_extra.sql`.

### Esquema

`audit_log(id, tabla, operacion, registro_id, antes_json, despues_json, usuario, timestamp)`.
`timestamp` usa `CURRENT_TIMESTAMP` de SQLite, que es **UTC**. Los JSON se
construyen con `json_object()` en los triggers y se normalizan con `serde_json`
antes de devolverse al frontend.

Hay triggers `AFTER INSERT`, `AFTER UPDATE` y `AFTER DELETE` en:
`trabajadores`, `recibos`, `periodos_nomina`, `incapacidades` y `vacaciones`.
Cada uno guarda el snapshot completo del registro antes y después del cambio.

### Usuario de la sesión

Se resuelve en este orden: variable de entorno `NOMINA_USUARIO` → clave
`NOMINA_USUARIO` o `usuario` en `~/.nomina-mx/config.json` (Windows:
`%APPDATA%\.nomina-mx\config.json`) → `"system"`. El comando
`configurar_usuario_auditoria` sincroniza ese valor apenas arranca la app y
`listar_auditoria` lo refresca en cada consulta. Los triggers leen el valor de
la tabla `audit_sesion`.

### Comando y filtros

```ts
import { invoke } from '@tauri-apps/api/core';

interface FiltrosAuditoria {
  tabla?: string;
  operacion?: string;
  usuario?: string;
  fecha_desde?: string; // YYYY-MM-DD, UTC
  fecha_hasta?: string; // YYYY-MM-DD, UTC
  limit?: number;
  offset?: number;
}

await invoke('configurar_usuario_auditoria');

const registros = await invoke('listar_auditoria', {
  filtros: {
    tabla: 'trabajadores',
    operacion: 'UPDATE',
    fecha_desde: '2026-11-01',
    fecha_hasta: '2026-11-30',
    limit: 10,
    offset: 0,
  },
});
```

### Inmutabilidad

`audit_log` solo admite `INSERT`. Los triggers `audit_log_no_update` y
`audit_log_no_delete` lanzan `RAISE(ABORT, 'audit_log es inmutable...')` ante
cualquier `UPDATE` o `DELETE`. El test `audit_log_no_se_puede_actualizar`
verifica ambos rechazos y que el número de filas no cambia.

### Consulta SQL de ejemplo

```sql
SELECT timestamp, tabla, operacion, registro_id, usuario, antes_json, despues_json
FROM audit_log
WHERE tabla = 'trabajadores'
  AND date(timestamp) BETWEEN '2026-11-01' AND '2026-11-30'
ORDER BY id DESC
LIMIT 10;
```

La UI `src/pages/Auditoria.tsx` muestra esta consulta en una tabla paginada con
filtros por tabla, operación, usuario y rango de fechas (UTC).

### Evidencia

`src-tauri/tests/auditoria.rs`: `trigger_insert_trabajador_crea_audit_log`,
`trigger_update_trabajador_guarda_antes_y_despues`,
`trigger_delete_trabajador_guarda_registro`,
`audit_log_no_se_puede_actualizar` y `listar_auditoria_filtra_por_tabla`.

## Multi-empresa

Estado: **Cubierto** (ÍTEM C4). Implementado en
`src-tauri/migrations/0004_multi_empresa.sql`, `src-tauri/src/commands/empresas.rs`
y `src/pages/EmpresaSelector.tsx`.
Referencias: LISR art. 79 fracc. III (requisitos de deducción de nómina) y
CFF art. 27 (contabilidad separada por establecimiento).

> Nota de versión: el archivo se llama `0004_multi_empresa.sql` y se registra
> como migración **5** en `lib.rs` (1 init, 2 seed fiscal, 3 horas extra,
> 4 auditoría, 5 multi-empresa).

### Esquema y migración

- `config(clave TEXT PRIMARY KEY, valor TEXT NOT NULL)` guarda la selección en
  `clave = 'empresa_activa'` con el ID como texto.
- `CREATE UNIQUE INDEX ux_empresas_rfc ON empresas(rfc)`: RFC único por empresa,
  reforzado en la base de datos.
- `empresa_id INTEGER NOT NULL DEFAULT 1` agregado a `recibos`,
  `incapacidades` y `vacaciones`. `trabajadores` y `periodos_nomina` ya lo
  tenían desde `0001_init.sql`.
- Se crea automáticamente la **Empresa Default** (`id = 1`,
  RFC `XAXX010101000`, razón social `Empresa Default`) si no existe.
- Todos los registros preexistentes se reasignan a `empresa_id = 1` mediante
  `UPDATE` explícito en las cinco tablas de datos.
- Los triggers de auditoría de `recibos`, `incapacidades` y `vacaciones` se
  recrean para incluir `empresa_id` en el snapshot JSON.

### Regla de negocio: la empresa activa se resuelve en servidor

`empresa_activa_pool()` lee `config.empresa_activa` (fallback a `1`). Ninguna
consulta confía en el `empresa_id` que envía el cliente:

| Comando | Comportamiento C4 |
| --- | --- |
| `listar_empresas` | Lista todas y marca `activa: true` en la seleccionada. |
| `cambiar_empresa_activa(id)` | Valida que exista y persiste el ID en `config`. |
| `crear_empresa(nueva)` | Valida RFC único y devuelve la empresa creada. |
| `guardar_trabajador` | Ignora `t.empresa_id`; sella la empresa activa (INSERT y UPDATE con `AND empresa_id=?`). |
| `listar_trabajadores` / `obtener_trabajador` | Filtran por la empresa activa. |
| `guardar_periodo` | Ignora `periodo.empresa_id`; sella la empresa activa. |
| `listar_periodos` / `listar_recibos` | Filtran por la empresa activa. |
| `guardar_recibo` | Escribe `empresa_id` con la activa. |
| `reporte_cuotas_imss` | Filtra `periodos_nomina` **y** `recibos` por la activa. |
| `obtener_empresa` | Devuelve la ficha de la empresa activa. |

Ejemplo desde el frontend:

```ts
await invoke('listar_empresas');            // [{ id, rfc, razon_social, activa, ... }]
await invoke('cambiar_empresa_activa', { id: 2 });
await invoke('crear_empresa', { nueva: { rfc: 'BBBB010101ABC', razon_social: 'Empresa Dos' } });
```

### UI

`src/pages/EmpresaSelector.tsx` se muestra en el header. Al cambiar de empresa
llama a `cambiar_empresa_activa` e incrementa `reloadKey` en Zustand
(`src/lib/store.ts`); `Trabajadores.tsx`, `Nomina.tsx`, `Reportes.tsx` y
`Empresa.tsx` recargan sus datos en consecuencia. El alta de empresa también
permite crear y activar una empresa nueva sin recurrir a SQL.

### Evidencia

`src-tauri/tests/multi_empresa.rs`: `multi_empresa_aislamiento_trabajadores`,
`multi_empresa_aislamiento_recibos`, `cambiar_empresa_activa_persiste`,
`crear_empresa_valida_rfc_unico` y
`migracion_0004_asigna_default_a_registros_existentes`.

## Recálculo retroactivo de SBC y recibos

Estado: **Cubierto** (ÍTEM C5). Implementado en
`src-tauri/src/commands/recalculo.rs`, registrado en el `invoke_handler` de
`src-tauri/src/lib.rs` y enlazado en `src/pages/Recalculo.tsx` (ruta
`/recalculo`).

### Fundamento y regla de oro

- Fuentes: CFF art. 28 (conservación de registros), LSS arts. 27 y 28 (SDI/SBC
  y tope de 25 UMA), LISR arts. 96 y 99 (ISR y subsidio), RMF Anexos 8 y 11
  (tarifas con vigencia) y CFDI 4.0 / Nómina 1.2 (comprobante timbrado
  inmutable).
- Los parámetros se resuelven con `cargar_param_vigente_pool(tipo,
  fecha_pago_original)` de la tabla `tablas_fiscales` (`vigencia_inicio` ≤
  fecha ≤ `vigencia_fin`). **Nunca se usa `CURRENT_DATE`**. Un periodo con
  `fecha_pago = 2026-01-31` usa UMA diaria **$113.14** y subsidio **$536.21**,
  aunque el recálculo se ejecute en noviembre 2026 (UMA $117.31 / subsidio
  $535.65).
- El motivo es **obligatorio** en `recalcular_periodo`. Vacío o solo espacios
  devuelve `Err`. Si el periodo no existe en la empresa activa, también `Err`.
- Se permiten periodos **abiertos o cerrados**; el estado se conserva y se
  copia al JSON de auditoría. El cierre no bloquea la corrección, pero el
  motivo deja constancia.
- Cada recálculo inserta una fila en `audit_log` con
  `operacion = 'RECALCULO'`, `tabla = 'recibos'`, `registro_id = periodo_id` y
  el motivo en `despues_json`, junto con la UMA y el subsidio usados.

### Flujo

1. Se carga el periodo por `id` y empresa activa (regla C4). Se toman
   `dias`, `fecha_pago` y `estado`.
2. Se leen los recibos del periodo con nombre del trabajador.
3. Para cada recibo se recalcula el SBC y el recibo completo con los parámetros
   vigentes a `fecha_pago`. Las horas extra se reconstruyen desde el desglose
   guardado (tiempo total; reparto determinista con los parámetros 2026).
4. Se comparan percepciones, deducciones y neto. Las diferencias se devuelven
   **por trabajador** con antes/después y `delta_neto = neto_despues −
   neto_antes`.
5. Solo se hace `UPDATE` de los recibos cambiados **sin `uuid_cfdi`**. Los
   timbrados se reportan en `diferencias` pero sus importes, `xml_cfdi` y
   `uuid_cfdi` quedan intactos.
6. Actualizaciones y fila de auditoría se confirman en una sola transacción.

### Comandos Tauri

```ts
interface Diferencia {
  trabajador_id: number;
  nombre: string;
  percepciones_antes: number;
  percepciones_despues: number;
  deducciones_antes: number;
  deducciones_despues: number;
  neto_antes: number;
  neto_despues: number;
  delta_neto: number;
}

interface ResultadoRecalculo {
  recibos_recalculados: number; // excluye CFDI timbrados
  diferencias: Diferencia[];
}

// Motivo obligatorio: recalcula todos los recibos del periodo.
const resultado = await invoke<ResultadoRecalculo>('recalcular_periodo', {
  periodoId: 12,
  motivo: 'Corrección de salario posterior al cierre',
});

// Firma sin motivo: usa motivo de sistema no vacío en audit_log.
const individual = await invoke<ResultadoRecalculo>(
  'recalcular_trabajador_periodo',
  { trabajadorId: 7, periodoId: 12 },
);
```

Equivalentes en `src/lib/tauri.ts`: `api.recalcularPeriodo(periodoId, motivo)`
y `api.recalcularTrabajadorPeriodo(trabajadorId, periodoId)`.

### Ejemplo numérico verificado

Con salario diario `10,000`, 30 días y `fecha_pago = 2026-01-31`:

- Factor de integración: `1 + (15 + 12×0.25 + 15×12×0.25/365)/365 =
  1.049651…`; SDI `10,496.51` pero se topa a `25 × 113.14 = 2,828.50`.
- Con UMA actual de noviembre sería `25 × 117.31 = 2,932.75`. El test
  `recalculo_usa_uma_historica` comprueba ambos valores y que la auditoría
  registra `uma_diaria = 113.14`.
- Con salario `350`, 30 días e ingreso `10,500`, ISR mensual
  `420.95 + (10,500 − 7,168.52)×10.88% = 783.415024`. El subsidio de enero
  (`536.21`) deja retención `247.205024`; el de febrero (`535.65`) deja
  `247.765024`. Diferencia exacta: **$0.56**.

### Advertencias

- No se modifica ningún recibo con `uuid_cfdi` no nulo: hacerlo dejaría el XML
  timbrado fuera de sincronía con la BD.
- No se genera CFDI de ajuste en esta entrega. `TipoDeComprobante="E"` queda
  registrado como **Pendiente v1.1** en `docs/pendientes_v1_1.md`.
- Las horas extra reconstruidas asumen jornada diurna, una semana y sin
  supuesto de salario mínimo porque esos datos no están persistidos en el
  recibo; para un caso atípico, revisar el XML original antes de aceptar el
  recálculo.

### Evidencia

`src-tauri/tests/recalculo.rs` cubre los cinco escenarios obligatorios sobre
BD temporal con migraciones 1–5: `recalculo_usa_uma_historica`,
`recalculo_con_subsidio_enero_usa_536_21`,
`recalculo_reporta_diferencias`, `recalculo_registra_auditoria` y
`recalculo_no_modifica_cfdi_timbrado`.
