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
Estado: **Pendiente para v1.1**. No existe tabla `audit_log` ni registro de usuario.
Los comandos actuales no registran autoría ni timestamp de cambio.

## Multi-empresa
Estado: **Parcial**. El esquema tiene `empresas` y `trabajadores.empresa_id`, pero la UI
fija `empresa_id = 1` en `Trabajadores.tsx` y `Nomina.tsx`. Selector de empresa: **Pendiente para v1.1**.

## Recálculo retroactivo de SBC
Estado: **Pendiente para v1.1**. No existe comando que recalcule `sdi`/SBC de periodos
anteriores ni que recalcule recibos ya guardados al cambiar UMA o prestaciones.
`calcular_recibo` calcula el periodo solicitado con los parámetros vigentes.
