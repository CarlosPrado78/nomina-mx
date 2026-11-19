# Respuesta técnica al reporte P1

**Fecha:** 2026-11-18
**Conclusión:** el checklist P1 no aplica a NominaMX.

## 1. Declaración de arquitectura

- NominaMX es una aplicación **Tauri 2 con backend Rust nativo**, no Tauri + Python.
- Se compila en un **binario único de 4.9 MB**:
  `src-tauri/target/release/nomina-mx`.
- La persistencia es **SQLite embebido** accedido con SQLx; las migraciones
  están en `src-tauri/migrations/` y se aplican con el runner propio de
  `src-tauri/src/db/mod.rs` (sin `tauri_plugin_sql`).
- La comunicación frontend↔backend es **IPC de Tauri** (`invoke` y
  `generate_handler!` en `src-tauri/src/lib.rs`), no HTTP.
- No hay runtime externo: el usuario final no necesita Python, Node ni Rust
  instalados en su máquina.

## 2. Evidencia de que P1 no aplica

Outputs literales del reporte de auditoría, que CONFIRMAN la ausencia de Python:

```
$ find . -type f \( -name '*.py' -o -name 'requirements*.txt' -o -name '*.spec' \) ...
(sin salida)
```

```
$ grep -rniE 'pyinstaller|nuitka|python3|backend_bin|backend\.exe|backend/main\.py|subprocess' --exclude-dir=node_modules --exclude-dir=.git --exclude-dir=target --exclude-dir=dist .
(sin salida)
```

No hay `bundle.resources` en `src-tauri/tauri.conf.json`; es correcto porque
no existe ningún binario externo que empaquetar:

```
  "bundle": {
    "active": true,
    "targets": ["dmg", "msi"],
    "icon": [ ... ]
  },
```

No hay `TcpListener` ni puerto: es IPC, no HTTP. Las únicas coincidencias de
la búsqueda son palabras como `importar`, `reportes`, `aportacion` o `importe`:

```
$ grep -rniE 'listen\(|TcpListener|127\.0\.0\.1|localhost:[0-9]|port' src-tauri/src
src-tauri/src/lib.rs:69: commands::importar::importar_trabajadores_csv,
src-tauri/src/lib.rs:82: commands::reportes::reporte_cuotas_imss,
...
```

El binario de 4.9 MB es la aplicación completa (Rust + SQLite + frontend):

```
$ du -h src-tauri/target/release/nomina-mx
4.9M    src-tauri/target/release/nomina-mx
```

## 3. Por qué la ausencia es una ventaja

- **Tamaño:** 4.9 MB frente a 30–50 MB de Tauri+Python y 80–150 MB de Electron.
- **Seguridad:** no hay servidor HTTP local ni puerto abierto; solo IPC de Tauri.
- **Simplicidad:** un binario, sin runtime, sin instalador de Python.
- **Rendimiento:** Rust nativo frente a Python interpretado.

## 4. Huecos reales que el reporte SÍ detecta

- **Verificación visual end-to-end en máquina limpia:** pendiente y
  **BLOQUEANTE** para v1.0. Detalle en `docs/smoke_test.md`, sección
  'Verificación visual end-to-end'. Hay una VM Windows 11 en Parallels, pero
  esta edición no permite `prlctl exec`, `capture` ni `send-key-event`, y el
  host tiene Python 3.14 y cargo, por lo que no es un entorno limpio.
- **Tag de release:** pendiente; se creará `v0.9.0` al cerrar el piloto.

## 5. Acción correctiva

- **NO** empaquetar Python: no aplica a esta arquitectura.
- **SÍ** ejecutar la verificación visual: intentada, registrada como
  BLOQUEANTE por falta de entorno limpio utilizable.
- **SÍ** crear el tag `v0.9.0` al cerrar v0.9.0 / v1.0.0. No se crea ahora
  porque la verificación visual no está cerrada.

Referencias: `docs/respuesta_p1.md`, `docs/smoke_test.md` y
`docs/pendientes_v1_1.md`.
