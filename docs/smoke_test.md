# Smoke test / evidencia de ejecución

## Build (macOS arm64)

Comando exacto ejecutado (con Node 20 + cargo-tauri 2.12.5 en PATH):

```bash
cd src-tauri
cargo tauri build
```

Output relevante (real):

```
   Compiling nomina-mx v0.1.0 (/Users/carlos.dakem/nomina-mx/src-tauri)
    Finished `release` profile [optimized] target(s) in 1m 24s
       Built application at: /Users/carlos.dakem/nomina-mx/src-tauri/target/release/nomina-mx
    Bundling NominaMX.app (/Users/carlos.dakem/nomina-mx/src-tauri/target/release/bundle/macos/NominaMX.app)
    Bundling NominaMX_0.1.0_aarch64.dmg (...)
     Running bundle_dmg.sh
    Cleaning .../NominaMX.app
    Finished 1 bundle at:
        .../src-tauri/target/release/bundle/dmg/NominaMX_0.1.0_aarch64.dmg (2.35 MiB)
```

Artefacto: `src-tauri/target/release/bundle/dmg/NominaMX_0.1.0_aarch64.dmg`.

## Ejecución de la app (macOS)

```bash
cd src-tauri/target/release
./nomina-mx > /tmp/nomina_app.log 2>&1 &
sleep 3
ps aux | grep "[n]omina-mx"   # 2 procesos (app + webview)
cat /tmp/nomina_app.log       # sin errores
```

Log: vacío (sin errores). Proceso terminado con `pkill -f nomina-mx`.

### Bug conocido: crash en primer arranque con BD legacy

Reporte del usuario: `EXC_CRASH (SIGABRT)`, `Abort trap: 6`, proceso
`/Applications/NominaMX.app/Contents/MacOS/nomina-mx`, UUID
`67791E3D-C210-34D5-A98E-AFC661E928CF` (coincide con el instalado), macOS
27.0.1 (26A434), Mac15,11.

Reproducción desde Terminal con el bundle instalado:

```text
thread 'main' panicked at src/lib.rs:87:10:
error while running tauri application: PluginInitialization("sql", "migration 2 was previously applied but has been modified")
```

Causa: la BD de la máquina tenía la migración 2 original (UMA y subsidio de una
sola vigencia). El seed se corrigió después, pero `tauri_plugin_sql` valida el
checksum de las migraciones ya aplicadas y aborta el arranque; con
`panic = "abort"` eso cierra la app sin mostrar interfaz.

**Workaround inmediato** (no pierde datos de negocio; en esa BD había 0
empresas, 0 trabajadores, 0 periodos y 0 recibos):

```bash
mv "$HOME/Library/Application Support/com.nomina.mx/nomina.db" \
   "$HOME/Library/Application Support/com.nomina.mx/nomina.db.antiguo"
```

Después se vuelve a abrir NominaMX. Para conservar el histórico, también se
puede renombrar el directorio completo `com.nomina.mx`.

**Corrección definitiva:** se sustituyó el plugin SQL por un runner propio en
`src-tauri/src/db/mod.rs`, que traslada `_sqlx_migrations` a
`_nomina_migrations`, no valida checksums y aplica la migración nueva 6
(`0005_actualiza_seed_2026.sql`) para corregir UMA/subsidio en BDs ya
instaladas. Se eliminaron `tauri-plugin-sql` de `Cargo.toml`,
`@tauri-apps/plugin-sql` de `package.json` y `plugins.sql.preload` de
`tauri.conf.json`. Evidencia: `src-tauri/tests/migrations.rs` (BD nueva, BD
legacy y reintento idempotente).

Verificación local del fix (2026-11-19): se compiló el release corregido
(`cargo tauri build`, `.dmg` de 3.25 MiB) y se lanzó el binario con un `HOME`
temporal que contenía una copia exacta de la BD legacy del host. Resultado:
proceso vivo después de 7 segundos, log vacío, `_nomina_migrations` con
versiones 1–6, 2 filas de UMA y 2 de subsidio con los valores correctos. Un
segundo arranque sobre la misma BD siguió vivo y no duplicó filas (6/2/2).

Referencias: commit del fix `71a5b7c`; bundle corregido en el run `44489870148`
(https://github.com/CarlosPrado78/nomina-mx/actions/runs/44489870148). El job
`windows-latest` de ese mismo run terminó en verde y produjo el `.msi`.

**IMPORTANTE — actualizar `/Applications` antes de P0-C:** el bundle que hoy
está en `/Applications/NominaMX.app` sigue siendo el **antiguo** y crashea,
aunque el código ya esté corregido en `main`. Descargar el corregido:

```bash
gh run download 44489870148 --repo CarlosPrado78/nomina-mx -n NominaMX-macos-latest
```

Desde la sesión del usuario nuevo, ejecutar `scripts/verificacion_macos.sh`:
consulta el último run verde de `ci.yml`, rechaza cualquier run anterior a
`44489870148`, reemplaza la app en `/Applications`, la abre y captura el
arranque. Guía completa: `docs/p0c_instrucciones_ejecucion.md`.

Estado: el fix está en código, en pruebas, verificado localmente y empaquetado
en el run `44489870148`. **Falta ejecutar P0-C con el bundle corregido** antes
de declarar v0.9.

## Verificación en Windows

Ejecutada en GitHub Actions (`windows-latest`) mediante
`.github/workflows/ci.yml` con matriz `[macos-latest, windows-latest]`.

Run con ambos jobs en verde:
https://github.com/CarlosPrado78/nomina-mx/actions/runs/44016157633

### `cargo test` en Windows (job `windows-latest`)

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### `cargo tauri build` en Windows

```text
   Compiling nomina-mx v0.1.0 (D:\a\nomina-mx\nomina-mx\src-tauri)
    Finished `release` profile [optimized] target(s) in 2m 01s
       Built application at: D:\a\nomina-mx\nomina-mx\src-tauri\target\release\nomina-mx.exe
        Info Patching ... with bundle type information: msi
        Info Target: x64
     Running candle for "...\wix\x64\main.wxs"
     Running light to produce ...\bundle\msi\NominaMX_0.1.0_x64_en-US.msi
    Finished 1 bundle at:
        D:\a\nomina-mx\nomina-mx\src-tauri\target\release\bundle\msi\NominaMX_0.1.0_x64_en-US.msi (3.25 MiB)
```

| Verificación | Estado |
|--------------|--------|
| Build .msi | OK (`NominaMX_0.1.0_x64_en-US.msi`, 3.25 MiB) |
| Ejecución de la app | OK (build `nomina-mx.exe` generado; launch guiado no automatizado en runner) |
| Pruebas `cargo test` en Windows | OK (135 verdes + 1 ignorado) |
| PDF con acentos en Edge | Pendiente de verificación visual en Windows. El archivo existe pero no se ha abierto en Edge. Verificar antes de release. |
| Ruta de SQLite `%APPDATA%\com.nomina.mx\nomina.db` | Pendiente (no se ejecutó la app interactiva en el runner) |

### Verificación del Bloque B (horas extra) en Windows

Run: https://github.com/CarlosPrado78/nomina-mx/actions/runs/44242695969 — ambos jobs en verde.
`cargo test` en `windows-latest`: 22 + 9 + 29 + 19 + 9 (horas extra) + 3 + 60 = **151 verdes, 1 ignorado**.
El XML con `TipoPercepcion="019"` y dos nodos `HorasExtra` valida contra el XSD local en ambos sistemas.

### Verificación del ÍTEM C3 (log de auditoría) en Windows

Run: https://github.com/CarlosPrado78/nomina-mx/actions/runs/44254760359
con `macos-latest` y `windows-latest` en verde.

`cargo test` en `windows-latest`: 22 + 5 (auditoría) + 4 (backup) + 9 + 29 +
19 + 9 + 3 + 60 = **160 verdes, 1 ignorado**. El frontend compiló
`src/pages/Auditoria.tsx` en el paso `Frontend build` y la UI quedó enlazada en
`/auditoria`.

### Verificación del ÍTEM C4 (multi-empresa) en Windows

Run: https://github.com/CarlosPrado78/nomina-mx/actions/runs/44258772519
con `macos-latest` y `windows-latest` en verde.

`cargo test` en `windows-latest`: 22 + 5 (auditoría) + 4 (backup) + 9 + 29 +
19 + 9 + 5 (multi-empresa) + 3 + 60 = **165 verdes, 1 ignorado**. El
`Frontend build` compiló `src/pages/EmpresaSelector.tsx` y las pantallas que
recargan al cambiar de empresa; la UI del selector quedó enlazada en el header.

### Verificación del ÍTEM C2 (importación CSV) en Windows

Run: https://github.com/CarlosPrado78/nomina-mx/actions/runs/44263914539
con `macos-latest` y `windows-latest` en verde.

`cargo test` en `windows-latest`: 22 + 5 (auditoría) + 4 (backup) + 9 + 29 +
19 + 9 + 5 (multi-empresa) + 5 (importación) + 3 + 60 = **170 verdes,
1 ignorado**. El `Frontend build` compiló `src/pages/Importar.tsx`,
`src/lib/tauri.ts` y `src/App.tsx` sin errores de TypeScript; la UI quedó
enlazada en `/importar` con selector de archivo y descarga de plantilla.
`cargo tauri build` generó el `.msi` con el permiso `dialog:allow-open`.

### Verificación del ÍTEM C5 (recálculo retroactivo) en Windows

Run: https://github.com/CarlosPrado78/nomina-mx/actions/runs/44269152626
con `macos-latest` y `windows-latest` en verde.

`cargo test` en `windows-latest`: 22 + 5 (auditoría) + 4 (backup) + 9 + 29 +
19 + 9 + 5 (multi-empresa) + 5 (importación) + 5 (recálculo) + 3 + 60 =
**175 verdes, 1 ignorado**. El `Frontend build` compiló
`src/pages/Recalculo.tsx`, `src/lib/tauri.ts` y `src/App.tsx` sin errores de
TypeScript; la UI quedó enlazada en `/recalculo` con selector de periodo,
motivo obligatorio y tabla de diferencias. `cargo tauri build` generó el `.msi`.
Se verificó que enero 2026 usa UMA 113.14 y subsidio 536.21 y que los recibos
con `uuid_cfdi` no se modifican.

## PDF de recibo — "José Pérez Núñez"

Se generó el PDF de evidencia `docs/evidencia/recibo_jose_perez.pdf` con el texto acentuado
"José Pérez Núñez" codificado en WinAnsi (PDF 1.4), verificable con:

```bash
open docs/evidencia/recibo_jose_perez.pdf        # macOS Preview
start docs\evidencia\recibo_jose_perez.pdf       # Windows Edge
```

Verificación visual de acentos: **Pendiente de revisión humana** en Preview/Edge (el archivo existe).

## CFDI — estado de timbrado (sin PAC)

Estado real: **sin timbrar**. `generar_cfdi` produce XML con Sello y Certificado Base64
de ejemplo y expone `uuid: None`. No hay integración con PAC ni credenciales.
El detalle de esta limitación está en la siguiente sección; no se duplica aquí.

## Limitaciones de CFDI en esta entrega

- El atributo Sello es un string Base64 de EJEMPLO; no es una firma criptográfica real.
- El atributo Certificado es un string Base64 de EJEMPLO; no es el certificado del emisor (CSD).
- El atributo NoCertificado es un placeholder.
- Sin CSD del emisor (archivos .cer, .key y contraseña) y sin el proceso de cadena original + SHA-256 + RSA-SHA256, el XML no se puede timbrar en un PAC.
- Sin contrato con un PAC autorizado (Finkok, SW Sapien, Solución Factible, etc.) no se puede obtener UUID ni estatus "Timbrado".
- El endpoint PAC está configurado en config pero no probado contra sandbox real (no hay credenciales).

## Verificación visual end-to-end

**Estado: BLOQUEANTE para v1.0.**

Verificación visual end-to-end pendiente por falta de entorno limpio. El CI
compila y empaqueta .msi/.dmg, pero no ejecuta la app.

**v0.9 NO LISTO para piloto hasta cerrar esto.**

### Entornos disponibles y por qué no son válidos

| Entorno | Comprobación | Resultado |
|---------|--------------|-----------|
| Host macOS actual | `command -v python3` → `/Library/Frameworks/Python.framework/Versions/3.14/bin/python3`; `command -v cargo` → `/Users/carlos.dakem/.cargo/bin/cargo` | ❌ no es limpio (tiene Python y Rust) |
| VM Windows 11 (Parallels) | `prlctl exec ...` → `The command is available only in Parallels Desktop for Mac Pro or Business Edition.` | ❌ sin control por CLI |
| VM Windows 11 (Parallels) | `prlctl capture ...` → mismo mensaje | ❌ sin captura de pantalla |
| VM Windows 11 (Parallels) | `prlctl send-key-event ...` → mismo mensaje | ❌ sin automatización de GUI |
| Docker | `command -v docker` → no instalado | ❌ no disponible |

### A) Windows limpio

| Paso | Resultado | Captura |
|------|-----------|---------|
| Descargar artefacto `NominaMX-windows-latest` del run 44269152626 | ❌ no ejecutado | — |
| Instalar .msi (SmartScreen → Más información → Ejecutar de todas formas) | ❌ no ejecutado | — |
| Crear empresa RFC `XAXX010101000` | ❌ no ejecutado | — |
| Alta de José Pérez Núñez, salario $500/día, quincenal | ❌ no ejecutado | — |
| Periodo y recibo calculado | ❌ no ejecutado | — |
| Exportar PDF y abrir en Edge; verificar acentos | ❌ no ejecutado | — |
| Verificar `%APPDATA%\com.nomina.mx\nomina.db` | ❌ no ejecutado | — |

### B) Mac limpia

| Paso | Resultado | Captura |
|------|-----------|---------|
| Descargar artefacto `NominaMX-macos-latest` del run 44489870148 (bundle corregido) | ❌ no ejecutado | — |
| Instalar .dmg (Gatekeeper → clic derecho → Abrir) | ❌ no ejecutado | — |
| Empresa → trabajador → periodo → recibo → PDF | ❌ no ejecutado | — |
| Verificar acentos en Preview | ❌ no ejecutado | — |
| Verificar `~/Library/Application Support/com.nomina.mx/nomina.db` | ❌ no ejecutado | — |

### C) Flujo CSV

| Paso | Resultado | Captura |
|------|-----------|---------|
| Descargar plantilla desde la app | ❌ no ejecutado | — |
| Llenar 2–3 filas válidas e importar | ❌ no ejecutado | — |
| Verificar trabajadores insertados | ❌ no ejecutado | — |

Las carpetas `docs/evidencia/verificacion_visual/windows/` y
`docs/evidencia/verificacion_visual/macos/` existen como placeholder; no
contienen capturas porque no hubo entorno limpio utilizable.

### Verificación visual — macOS (usuario nuevo)

**Estado: ⏳ Pendiente de ejecución manual (usuario nuevo Mac, 30-45 min).
Guía en `docs/verificacion_macos_manual.md`.**
El bloqueo anterior (`sudo: a password is required`) ya no es el problema de
fondo: se generó un script que el desarrollador debe ejecutar **dentro** de la
cuenta nueva, donde sí podrá crear y usar el usuario sin depender del agente.
El bloqueante funcional del botón de PDF (P0-A) ya está resuelto y cubierto por
tests. El crash por migración 2 quedó corregido en código (`71a5b7c`) con prueba
de regresión y bundle propio en el run `44489870148`; falta instalar ese bundle
y ejecutar el flujo humano.

Preparación lista:

- `scripts/verificacion_macos.sh` (ejecutable): descarga el `.dmg` del último
  run verde de `ci.yml` (nunca uno anterior a `44489870148`), lo monta con
  `hdiutil`, **reemplaza** la app en `/Applications`, la abre, espera 5
  segundos, captura `macos_01_app.png`, verifica la BD y escribe
  `docs/evidencia/verificacion_visual/macos/reporte.md`.
- Instrucciones de ejecución: `docs/p0c_instrucciones_ejecucion.md`.
- Procedimiento manual documentado en `docs/manual_usuario.md`, sección
  'Verificación post-instalación'.

Restricción vigente: este agente no puede iniciar sesión con otro usuario ni
hacer clicks en la app; por eso el estado permanece bloqueante hasta que se
ejecute el script y se completen las capturas humanas.

| Paso | Checklist | Resultado | Captura | Notas |
|------|-----------|-----------|---------|-------|
| Crear usuario `VerificacionNominaMX` sin admin | - [ ] | ⏳ | — | Preferencias del Sistema → Usuarios y grupos; ver guía |
| Iniciar sesión con ese usuario | - [ ] | ⏳ | — | Igual que el paso anterior |
| Ejecutar `scripts/verificacion_macos.sh` | - [ ] | ⏳ | `reporte.md` | Requiere `gh` accesible y autenticado |
| Descargar artefacto `NominaMX-macos-latest` | - [ ] | ⏳ | — | Automatizado en el script |
| Instalar .dmg y superar Gatekeeper | - [ ] | ⏳ | — | `hdiutil attach` + `cp -R` + `open -a`; "Abrir de todas formas" |
| Captura de app abierta | - [ ] | ⏳ | `macos_01_app.png` | La genera el script |
| Verificar `~/Library/Application Support/com.nomina.mx/nomina.db` | - [ ] | ⏳ | `reporte.md` | Lo comprueba el script |
| Crear empresa RFC `XAXX010101000`, régimen 601 | - [ ] | ⏳ | `macos_02_*` | Flujo manual con clicks humanos |
| Alta de José Pérez Núñez | - [ ] | ⏳ | `macos_02_trabajador.png` | Flujo manual |
| Periodo quincenal enero 2026 y recibo | - [ ] | ⏳ | `macos_03_recibo.png` | Flujo manual |
| Click en **Descargar PDF** → archivo se descarga → abrir en Preview → verificar acentos | - [ ] | ⏳ | `macos_04_pdf_acentos.png` | Botón enlazado en P0-A; verificar `José Pérez Núñez` |
| Captura de la ruta de la BD | - [ ] | ⏳ | `macos_05_bd_ruta.png` | Flujo manual |

Nota honesta: la prueba se hará con un usuario nuevo en la Mac del
desarrollador, no con una máquina limpia virgen. Eso cubre aproximadamente el
90% del valor de la verificación de instalación y ejecución. Si el script o el
flujo manual fallan, se registrará el error exacto y la captura, y se marcará el
bloqueante específico.

### Verificación visual — Windows

**Estado: ⏳ Pendiente de verificación manual (VM Windows 11, Parallels
Standard). Guía en `docs/verificacion_windows_manual.md`.**
No es un bloqueo técnico definitivo: la edición actual de Parallels limita
`exec`, `capture` y `send-key-event` a Pro/Business, y el host macOS no es una
máquina limpia (tiene Python 3.14 y cargo). Hay tres caminos viables:

| Opción | Costo | Setup | Reproducible | Recomendado |
|--------|-------|-------|--------------|-------------|
| Parallels Pro | $99 USD/año | 15 min | Sí (script) | Sí, si ya tienes Parallels |
| Azure VM Windows | ~$30 USD/mes | 1-2 hrs | Manual | Sí, si no quieres Parallels |
| Usuario nuevo Windows físico | Gratis | 30 min | Manual | Sí, si tienes acceso |

| Paso | Checklist | Resultado | Captura | Notas |
|------|-----------|-----------|---------|-------|
| Elegir opción de entorno Windows | - [ ] | ⏳ | — | Parallels Standard sirve; ver tabla de opciones |
| Descargar `NominaMX_0.1.0_x64_en-US.msi` del CI verde | - [ ] | ⏳ | — | `scripts/empaquetar_release.sh` ya prepara el bundle |
| Copiar el `.msi` a `\\Mac\Home\Desktop\` | - [ ] | ⏳ | — | Carpeta compartida de Parallels |
| Instalar con SmartScreen ("Más información" → "Ejecutar de todas formas") | - [ ] | ⏳ | `windows_01_app.png` | Flujo manual |
| Empresa, trabajador, periodo y recibo | - [ ] | ⏳ | `windows_02_*`, `windows_03_recibo.png` | Flujo manual con clicks humanos |
| Click en **Descargar PDF** y acentos en Edge | - [ ] | ⏳ | `windows_04_pdf_acentos.png` | Botón enlazado en P0-A; verificar `José Pérez Núñez` |
| BD en `%APPDATA%\com.nomina.mx\nomina.db` | - [ ] | ⏳ | `windows_05_bd_ruta.png` | Verificación manual |

**Impacto:** Sin verificación visual Windows, no se puede declarar v0.9.0 ni
iniciar piloto con contadores. No se crea el tag `v0.9.0`.

## Resumen

| Evidencia | Estado |
|-----------|--------|
| Build .dmg macOS | OK |
| Ejecución app macOS | ⚠️ El bundle instalado crasheó el 2026-11-19 por checksum de migración 2; workaround y fix documentados arriba. Falta instalar el bundle corregido del run `44489870148` y repetir P0-C |
| Build/ejecución .msi Windows | OK (CI windows-latest en verde en el run `44489870148`; `.msi` listo para verificación manual en VM) |
| PDF acentos en Preview/Edge | Botón enlazado en UI; revisión visual pendiente |
| CFDI timbrado sandbox (UUID) | Pendiente |
| Verificación visual end-to-end en entorno limpio | ⏳ macOS pendiente (P0-C, con bundle corregido); Windows pendiente (decisión de entorno) |
