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
Estado: **Parcial**. El esquema tiene `empresas` y `trabajadores.empresa_id`, pero la UI
fija `empresa_id = 1` en `Trabajadores.tsx` y `Nomina.tsx`. Selector de empresa: **Pendiente para v1.1**.

## Recálculo retroactivo de SBC
Estado: **Pendiente para v1.1**. No existe comando que recalcule `sdi`/SBC de periodos
anteriores ni que recalcule recibos ya guardados al cambiar UMA o prestaciones.
`calcular_recibo` calcula el periodo solicitado con los parámetros vigentes.
