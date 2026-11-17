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

No hay VM Windows en el entorno de desarrollo (macOS arm64). El repositorio local
sí fue inicializado (commit `a37fef2`) y contiene el workflow, pero no tiene remoto
configured ni credenciales (`gh` no instalado), por lo que aún no se puede pegar un
URL de GitHub Actions con el job en verde. Se dejó configurado
`.github/workflows/ci.yml` con matriz `[macos-latest, windows-latest]` y pasos:
`cargo test`, `cargo build --release`, `npm ci && npm run build` y `cargo tauri build`.

Artefacto esperado en Windows:
`src-tauri/target/release/bundle/msi/NominaMX_0.1.0_x64_en-US.msi`.

| Verificación | Estado |
|--------------|--------|
| Build .msi | Pendiente (workflow configurado, sin ejecutar) |
| Ejecución de la app | Pendiente (sin VM Windows) |
| Pruebas `cargo test` en Windows | Pendiente (workflow configurado, sin ejecutar) |
| PDF con acentos en Edge | Pendiente de verificación visual en Windows. El archivo existe pero no se ha abierto en Edge. Verificar antes de release. |
| Ruta de SQLite `%APPDATA%\com.nomina.mx\nomina.db` | Pendiente (sin VM Windows) |

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

## Resumen

| Evidencia | Estado |
|-----------|--------|
| Build .dmg macOS | OK |
| Ejecución app macOS | OK (sin errores en log) |
| Build/ejecución .msi Windows | Pendiente |
| PDF acentos en Preview/Edge | Archivo generado; revisión visual pendiente |
| CFDI timbrado sandbox (UUID) | Pendiente |
