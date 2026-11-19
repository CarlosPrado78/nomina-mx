# P0-C — Instrucciones de ejecución (bundle corregido)

Guía para el desarrollador. Este agente **no puede** iniciar sesión con otro
usuario ni hacer clicks en la app: P0-C requiere ejecución manual.

- Fix del crash de migración: commit `71a5b7c`
  (`fix(migrations): reemplazar plugin SQL y corregir seed 2026`).
- Bundle corregido: run de CI `44489870148` —
  https://github.com/CarlosPrado78/nomina-mx/actions/runs/44489870148
- CI Windows de ese mismo run: **verde**. `.msi` listo para verificación manual en VM.
- **Importante:** el bundle hoy instalado en `/Applications/NominaMX.app` es el
  **antiguo** y crashea en el primer arranque con BD legacy. Hay que reemplazarlo
  antes de empezar; no basta con que `main` ya tenga el fix.

## 1. Descargar el `.dmg` corregido del run 44489870148

```bash
cd /ruta/a/nomina-mx
gh run download 44489870148 --repo CarlosPrado78/nomina-mx -n NominaMX-macos-latest
```

Si `gh` no está en el PATH, use la ruta absoluta
(`/Users/carlos.dakem/bin/gh`) o exporte `GH_BIN=/ruta/a/gh` y `GH_TOKEN=...`.

Comprobar que el artefacto descargado contiene
`NominaMX-macos-latest/NominaMX_0.1.0_aarch64.dmg`. No use un run anterior a
`44489870148`: esos bundles tienen el bug de checksum de migración.

## 2. Reemplazar el bundle en `/Applications`

La forma recomendada es dejar que `scripts/verificacion_macos.sh` (paso 3) haga
la descarga, el montaje y el reemplazo. Si decide instalar manualmente:

```bash
DMG='NominaMX-macos-latest/NominaMX_0.1.0_aarch64.dmg'
MONTURA="$(mktemp -d)/montura"
mkdir -p "$MONTURA"
hdiutil attach "$DMG" -mountpoint "$MONTURA" -nobrowse
cp -R /Applications/NominaMX.app "$HOME/NominaMX.app.anterior" 2>/dev/null || true
rm -rf /Applications/NominaMX.app
cp -R "$MONTURA/NominaMX.app" /Applications/
hdiutil detach "$MONTURA"
```

No use un simple `cp -R` sobre el `.app` existente: fusiona directorios y puede
dejar el binario antiguo. Si no hay permisos para escribir en `/Applications`,
no continúe: documente el error exacto. El script registrará un fallback ⚠️ con
una copia temporal, y eso **no** cuenta como P0-C terminado.

## 3. Pasos exactos de P0-C

1. Crear el usuario estándar (no administrador) `VerificacionNominaMX` en
   **Preferencias del Sistema → Usuarios y grupos**, sin Homebrew, Python, cargo
   ni Node.
2. Cerrar sesión e iniciar sesión con ese usuario.
3. Desde su sesión, situarse en el repo y ejecutar el script:

   ```bash
   cd /ruta/a/nomina-mx
   GH_BIN=/ruta/a/gh GH_TOKEN=... scripts/verificacion_macos.sh
   ```

   El script consulta el último run verde de `ci.yml`, se niega a instalar un
   run anterior a `44489870148`, descarga `NominaMX-macos-latest`, monta el
   `.dmg` con `hdiutil`, **reemplaza** `/Applications/NominaMX.app`, abre la app,
   espera 5 s, captura `macos_01_app.png` y verifica
   `~/Library/Application Support/com.nomina.mx/nomina.db`. Resultado en
   `docs/evidencia/verificacion_visual/macos/reporte.md`.
4. Si Gatekeeper lo impide: clic derecho sobre **NominaMX.app → Abrir**, o
   **Preferencias del Sistema → Seguridad y Privacidad → Abrir de todas formas**.
5. Flujo manual con clicks humanos (el script no lo automatiza):
   - **Empresa:** RFC `XAXX010101000`, régimen `601`, guardar.
   - **Trabajadores:** José Pérez Núñez, CURP `PENJ850101HDFRRN01`, RFC
     `PENJ850101AB1`, NSS `12345678901`, ingreso `2026-01-15`, salario diario
     `500.00`, periodicidad quincenal, contrato indeterminado.
   - **Nómina:** periodo quincenal de enero 2026, fecha de pago en enero,
     seleccionar al trabajador y **Calcular nómina**.
   - Clic en **Descargar PDF** en la última columna del renglón.
   - Abrir el PDF en **Preview** y verificar `José Pérez Núñez` con acentos.
   - En Terminal: `ls -la ~/Library/Application\ Support/com.nomina.mx/`.
6. Capturas con **Cmd+Shift+4/5** y guárdelas en
   `docs/evidencia/verificacion_visual/macos/`: `macos_01_app.png` (script),
   `macos_02_trabajador.png`, `macos_03_recibo.png`,
   `macos_04_pdf_acentos.png`, `macos_05_bd_ruta.png`.
7. Cerrar la app manualmente (el script no la cierra).

## 4. Checklist

- [ ] Bundle del run `44489870148` (o posterior) descargado.
- [ ] `/Applications/NominaMX.app` **reemplazado** (no solo `cp -R` encima).
- [ ] Usuario `VerificacionNominaMX` creado como estándar y sesión iniciada.
- [ ] `scripts/verificacion_macos.sh` ejecutado; `reporte.md` sin filas ❌ ni ⚠️.
- [ ] Gatekeeper superado ("Abrir de todas formas").
- [ ] `macos_01_app.png` generada por el script.
- [ ] Empresa `XAXX010101000`, régimen `601` creada.
- [ ] Trabajador José Pérez Núñez dado de alta con los datos indicados.
- [ ] Periodo quincenal de enero 2026 calculado.
- [ ] Botón **Descargar PDF** descargó `recibo_<id>.pdf`.
- [ ] PDF abierto en Preview con acentos correctos.
- [ ] `nomina.db` verificado en `~/Library/Application Support/com.nomina.mx/`.
- [ ] Capturas `macos_02` a `macos_05` guardadas en el repo.
- [ ] `docs/smoke_test.md` actualizado con los resultados reales.
- [ ] App cerrada manualmente.

## 5. Si algo falla

No escriba "no funciona". Pegue el **error específico** y añada: comando
ejecutado, salida completa, ID del run y nombre del artefacto, `sw_vers`, la
captura del fallo y, si hay crash, el UUID del informe.

| Síntoma | Acción |
|---------|--------|
| `migration 2 was previously applied but has been modified` / `SIGABRT` | El bundle es antiguo. Repita el paso 2 con el run `44489870148`. Como último recurso, renombre (no borre) `~/Library/Application Support/com.nomina.mx/nomina.db` a `nomina.db.antiguo` y vuelva a abrir. Ver `docs/smoke_test.md`. |
| `cp -R ... /Applications:` sin permisos / ⚠️ en `reporte.md` | Anote el error exacto; no marque P0-C como terminado hasta que el bundle esté realmente en `/Applications`. |
| `gh: command not found` | Exporte `GH_BIN=/Users/carlos.dakem/bin/gh` y `GH_TOKEN=...`. |
| `screencapture` sin permisos | Preferencias del Sistema → Privacidad y seguridad → Grabación de pantalla: agregar Terminal. |
| PDF con acentos rotos (`JosÃ©`) | Adjunte `macos_04_pdf_acentos.png` y reporte como bloqueante específico. |
| No se crea `nomina.db` | Verifique permisos de `ls -ld ~/Library/Application\ Support/com.nomina.mx/` y anote la salida. |

## 6. Al terminar

1. Actualizar `docs/smoke_test.md`, sección "Verificación visual — macOS
   (usuario nuevo)", con ✅/❌ por paso, las rutas de las cinco capturas y el
   contenido real de `reporte.md`.
2. Commitear y pushear el resultado con un mensaje descriptivo, por ejemplo:

   ```bash
   git add -A
   git commit -m "docs: P0-C ejecutado con bundle corregido"
   git push origin main
   ```

3. **No** crear el tag `v0.9.0`, no publicar Release ni iniciar piloto con
   contadores hasta tener P0-C ✅ **y** P1 ✅ (verificación manual de Windows en
   VM, `.msi` del run `44489870148`).
