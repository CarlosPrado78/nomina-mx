# Verificación manual de macOS — NominaMX v0.9 (P0-C)

Esta guía es para el desarrollador. Comprueba que el instalador funciona en un
**usuario nuevo de macOS**, sin las herramientas de desarrollo instaladas en la
cuenta habitual. No requiere Parallels.

## Requisitos

- macOS 11 o superior en la Mac del desarrollador.
- Usuario estándar (no administrador) `VerificacionNominaMX`, sin Homebrew,
  Python, cargo ni Node. Solo aplicaciones del sistema.
- El repositorio `nomina-mx` clonado en una ruta legible por ese usuario.
- `gh` accesible y autenticado. No es un runtime de desarrollo: es solo el CLI
  que usa el script para descargar el artefacto del CI. Si no está en el PATH,
  exporte `GH_BIN=/ruta/a/gh` y `GH_TOKEN=su_token`.
- No se necesita Parallels ni ninguna VM.

## Pasos exactos

1. **Crear el usuario.** En **Preferencias del Sistema → Usuarios y grupos**,
   pulse **+**, nombre completo `VerificacionNominaMX`, tipo de cuenta
   **Estándar**. No marque "Permitir administrar". No instale nada más en esa
   cuenta.
2. **Cerrar sesión** e **iniciar sesión** como `VerificacionNominaMX`.
3. Abrir **Terminal** y situarse en el repo:

   ```bash
   cd /ruta/a/nomina-mx
   GH_BIN=/usr/local/bin/gh GH_TOKEN=su_token scripts/verificacion_macos.sh
   ```

   El script: elige el último run verde de `main`, descarga
   `NominaMX-macos-latest`, monta el `.dmg` con `hdiutil`, copia
   `NominaMX.app` a `/Applications`, la abre, espera 5 segundos, guarda
   `macos_01_app.png`, verifica
   `~/Library/Application Support/com.nomina.mx/nomina.db` y escribe
   `docs/evidencia/verificacion_visual/macos/reporte.md`.
4. **Gatekeeper:** si la app no abre, haga clic derecho sobre
   **NominaMX.app → Abrir**, o vaya a **Preferencias del Sistema → Seguridad y
   Privacidad → Abrir de todas formas**. Solo se hace una vez.
5. **Flujo manual con clicks humanos** (el script NO lo automatiza):
   - En **Empresa**: RFC `XAXX010101000`, régimen `601`, guardar.
   - En **Trabajadores**: José Pérez Núñez, CURP `PENJ850101HDFRRN01`, RFC
     `PENJ850101AB1`, NSS `12345678901`, ingreso `2026-01-15`, salario diario
     `500.00`, periodicidad quincenal, contrato indeterminado.
   - En **Nomina**: periodo quincenal de enero 2026, fecha de pago dentro de
     enero, seleccionar al trabajador y **Calcular nomina**.
   - Hacer clic en **Descargar PDF** en la última columna del renglón.
   - Abrir el PDF en **Preview** y verificar `José Pérez Núñez` con acentos.
   - En Terminal: `ls -la ~/Library/Application\ Support/com.nomina.mx/`.
6. **Capturas:** use **Cmd+Shift+4** (o **Cmd+Shift+5**) y guarde temporalmente
   en el Escritorio. Luego mueva los archivos al repo:

   ```bash
   mv ~/Desktop/macos_0*.png docs/evidencia/verificacion_visual/macos/
   ```

   Nombres esperados: `macos_01_app.png` (la genera el script),
   `macos_02_trabajador.png`, `macos_03_recibo.png`,
   `macos_04_pdf_acentos.png`, `macos_05_bd_ruta.png`.
7. **Cerrar la app manualmente.** El script no la cierra.
8. **Registrar resultados** en `docs/smoke_test.md`, sección
   "Verificación visual — macOS (usuario nuevo)", con ✅/❌ por paso. Si hay un
   fallo, documentar el error exacto y la captura.

## Checklist

- [ ] Usuario `VerificacionNominaMX` creado como estándar (sin admin).
- [ ] Sesión iniciada con ese usuario; sin Homebrew, Python, cargo ni Node.
- [ ] `scripts/verificacion_macos.sh` ejecutado y `reporte.md` generado.
- [ ] `.dmg` descargado del último CI verde e instalado en `/Applications`.
- [ ] Gatekeeper superado ("Abrir de todas formas").
- [ ] Empresa `XAXX010101000`, régimen `601` creada.
- [ ] Trabajador José Pérez Núñez dado de alta con los datos indicados.
- [ ] Periodo quincenal de enero 2026 calculado.
- [ ] Botón **Descargar PDF** descargó `recibo_<id>.pdf`.
- [ ] PDF abierto en Preview con acentos correctos.
- [ ] `nomina.db` verificado en la ruta de Application Support.
- [ ] Cinco capturas guardadas en `docs/evidencia/verificacion_visual/macos/`.
- [ ] `docs/smoke_test.md` actualizado con resultados reales.
- [ ] App cerrada manualmente.

## Errores comunes

| Síntoma | Cómo resolverlo |
|---------|-----------------|
| Gatekeeper impide abrir ("app dañada" o no abre) | Clic derecho → **Abrir**, o **Preferencias del Sistema → Seguridad y Privacidad → Abrir de todas formas**. |
| "NominaMX se cerró inesperadamente" apenas se abre | Asegúrese de que el `.dmg` provenga de un run de CI **posterior a este fix**. Si venía de un bundle anterior, instale el nuevo; como workaround temporal renombre `~/Library/Application Support/com.nomina.mx/nomina.db` a `nomina.db.antiguo` y vuelva a abrir. Ver `docs/smoke_test.md`, sección del incidente 2026-11-19. |
| `gh: command not found` | Exportar `GH_BIN=/ruta/a/gh` y `GH_TOKEN=...`. Como alternativa, descargar el bundle con `scripts/empaquetar_release.sh` en la cuenta del desarrollador e instalar el `.dmg` manualmente; el script de verificación **no acepta** un `.dmg` local. |
| `screencapture` falla | **Preferencias del Sistema → Privacidad y seguridad → Grabación de pantalla**: agregar Terminal. |
| PDF muestra `JosÃ©` o acentos rotos | Adjuntar la captura y reportar como **bloqueante específico**: posible bug de encoding de `@react-pdf/renderer` en macOS. |
| No se crea `nomina.db` | Verificar permisos: `ls -ld ~/Library/Application\ Support/com.nomina.mx/`. Si la carpeta no existe o no pertenece al usuario, crearla con `mkdir -p` desde esa cuenta y volver a abrir la app. Si el script avisó "sin permisos; se usa copia temporal", revisar también esa ruta. |
| `cp -R ... /Applications/` sin permisos | El script usa una copia temporal y lo registra como ⚠️ en `reporte.md`; anotarlo en `smoke_test.md`. |

**Resultado esperado:** macOS queda "✅ Verificado (usuario nuevo, no máquina
virgen)". Si algún paso falla, se documenta el error exacto y se mantiene como
bloqueante específico. No se declara `v0.9.0` mientras Windows siga pendiente.
