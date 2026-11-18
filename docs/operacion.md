# Operación

## Importación masiva de trabajadores (CSV/Excel)
Estado: **Pendiente para v1.1**. No existe comando Tauri ni pantalla de importación.
El alta es individual en `src/pages/Trabajadores.tsx` vía `guardar_trabajador`.

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

## Recálculo retroactivo de SBC
Estado: **Pendiente para v1.1**. No existe comando que recalcule `sdi`/SBC de periodos
anteriores ni que recalcule recibos ya guardados al cambiar UMA o prestaciones.
`calcular_recibo` calcula el periodo solicitado con los parámetros vigentes.
