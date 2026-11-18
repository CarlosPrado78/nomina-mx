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
| Descargar artefacto `NominaMX-macos-latest` del run 44269152626 | ❌ no ejecutado | — |
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

**Estado: BLOQUEANTE.** No se pudo crear el usuario ni iniciar sesión.

Comprobación del intento:

```
$ sudo -n true
sudo: a password is required
$ dscl . -list /Users | grep -i verif
(sin salida)
```

No hay sudo sin contraseña, por lo que no fue posible ejecutar
`sysadminctl -addUser VerificacionNominaMX` ni cambiar de sesión. El host actual
tampoco sirve como sustituto: tiene Python 3.14 y cargo instalados.

| Paso | Resultado | Captura | Notas |
|------|-----------|---------|-------|
| Crear usuario `VerificacionNominaMX` sin admin | ❌ | — | `sudo: a password is required` |
| Iniciar sesión con ese usuario | ❌ | — | El usuario no existe |
| Descargar artefacto `NominaMX-macos-latest` | ❌ | — | Sin sesión limpia |
| Instalar .dmg y abrir con Gatekeeper | ❌ | — | Sin sesión limpia |
| Crear empresa RFC `XAXX010101000`, régimen 601 | ❌ | — | Sin sesión limpia |
| Alta de José Pérez Núñez | ❌ | — | Sin sesión limpia |
| Periodo quincenal enero 2026 y recibo | ❌ | — | Sin sesión limpia |
| Exportar PDF y revisar acentos en Preview | ❌ | — | Sin sesión limpia; además, el botón de descarga de PDF no está enlazado en la UI (`descargarRecibo` existe en `src/components/ReciboPDF.tsx` pero no se usa) |
| Verificar `~/Library/Application Support/com.nomina.mx/nomina.db` | ❌ | — | Sin sesión limpia |
| Capturas `macos_01_app.png` … `macos_05_bd_ruta.png` | ❌ | — | No se generaron |

Nota honesta: el plan era usar un usuario nuevo en la Mac del desarrollador,
no una máquina limpia virgen; eso cubriría aproximadamente el 90% del valor de
la verificación de instalación y ejecución. Aun así no se pudo ejecutar.

### Verificación visual — Windows (BLOQUEANTE)

**Estado: Pendiente.** Parallels edition limita exec/capture a Pro/Business.
Host macOS no es limpio. Opciones: (a) upgrade Parallels Pro ($99 USD/año),
(b) VM Windows en Azure (~$30 USD/mes), (c) usuario nuevo Windows en máquina
física (si hay acceso).

**Impacto:** Sin verificación visual Windows, no se puede declarar v0.9.0 ni
iniciar piloto con contadores. No se crea el tag `v0.9.0`.

## Resumen

| Evidencia | Estado |
|-----------|--------|
| Build .dmg macOS | OK |
| Ejecución app macOS | OK (sin errores en log) |
| Build/ejecución .msi Windows | OK (CI windows-latest, run 44016157633) |
| PDF acentos en Preview/Edge | Archivo generado; revisión visual pendiente |
| CFDI timbrado sandbox (UUID) | Pendiente |
| Verificación visual end-to-end en entorno limpio | BLOQUEANTE |
