# Operación

## Importación masiva de trabajadores (CSV/Excel)
Estado: **Pendiente para v1.1**. No existe comando Tauri ni pantalla de importación.
El alta es individual en `src/pages/Trabajadores.tsx` vía `guardar_trabajador`.

## Backup / restore de la BD
Estado: **Pendiente para v1.1** (sin comando en la app). Procedimiento manual con sqlite3:

```bash
# La BD está en app data dir:
# macOS: ~/Library/Application Support/com.nomina.mx/nomina.db
# Windows: %APPDATA%\com.nomina.mx\nomina.db

sqlite3 nomina.db ".backup nomina_backup_2026.db"
sqlite3 nomina_restaurada.db < dump.sql
```

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
