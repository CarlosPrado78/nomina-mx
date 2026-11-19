# Verificación manual de Windows — NominaMX v0.9 (P1)

Esta guía permite verificar el instalador de Windows sin Parallels Pro: usa la
edición **Standard** y la carpeta compartida que ya ofrece Parallels. No se
necesita `prlctl`.

## Requisitos

- macOS host con **Parallels Desktop Standard** y una **VM Windows 11 en
  ejecución** (Windows 10 o superior también sirve).
- Compartición de la carpeta de inicio de macOS habilitada en Parallels
  (configuración por defecto: `\\Mac\Home\`).
- `gh` autenticado en el host para descargar el artefacto del CI, o el `.msi`
  ya descargado en `dist/`.
- Herramienta de captura de Windows (**Win+Shift+S** o Recortes).

## Pasos exactos

1. **Descargar el instalador en el host macOS:**

   ```bash
   cd /ruta/a/nomina-mx
   scripts/empaquetar_release.sh
   cp dist/NominaMX_0.1.0_x64_en-US.msi ~/Desktop/
   ```

   El script toma el último run verde de `main` y deja ambos bundles en
   `dist/`. Si solo se quiere Windows:
   `gh run download <RUN_ID> --repo CarlosPrado78/nomina-mx --name NominaMX-windows-latest`.

2. **Copiar el `.msi` a la VM.** Dentro de Windows, abrir el Explorador de
   archivos y navegar a:

   ```
   \\Mac\Home\Desktop\
   ```

   Debe aparecer `NominaMX_0.1.0_x64_en-US.msi`. Si no aparece, revisar la
   carpeta compartida de Parallels.

3. **Instalar.** Doble clic en el `.msi` y seguir el asistente.
4. **SmartScreen:** si aparece "Windows protegió su PC", clic en
   **Más información → Ejecutar de todas formas**. Solo se hace una vez.
5. **Abrir NominaMX** desde el menú Inicio.
6. **Flujo manual con clicks humanos:**
   - En **Empresa**: RFC `XAXX010101000`, régimen `601`, guardar.
   - En **Trabajadores**: José Pérez Núñez, CURP `PENJ850101HDFRRN01`, RFC
     `PENJ850101AB1`, NSS `12345678901`, ingreso `2026-01-15`, salario diario
     `500.00`, periodicidad quincenal, contrato indeterminado.
   - En **Nomina**: periodo quincenal de enero 2026, fecha de pago en enero,
     seleccionar al trabajador y **Calcular nomina**.
   - Clic en **Descargar PDF** en la última columna del renglón.
   - Abrir el PDF en **Edge** y verificar `José Pérez Núñez` con acentos.
   - Verificar la BD en PowerShell o Explorador:

     ```powershell
     dir "$env:APPDATA\com.nomina.mx\nomina.db"
     ```

7. **Capturas:** pulsar **Win+Shift+S**, elegir la región y guardar en
   `\\Mac\Home\Desktop\`. Luego moverlas en el host macOS:

   ```bash
   mv ~/Desktop/windows_0*.png docs/evidencia/verificacion_visual/windows/
   ```

   Nombres esperados: `windows_01_app.png`, `windows_02_trabajador.png`,
   `windows_03_recibo.png`, `windows_04_pdf_acentos.png`,
   `windows_05_bd_ruta.png`.
8. **Actualizar** `docs/smoke_test.md`, sección "Verificación visual —
   Windows", con ✅/❌ por paso. Anotar el error exacto y la captura si algo
   falla.

## Checklist

- [ ] VM Windows 11 en ejecución en Parallels Standard.
- [ ] `.msi` del último CI verde descargado en `dist/` y copiado a
      `~/Desktop/`.
- [ ] `.msi` visible en `\\Mac\Home\Desktop\` desde la VM.
- [ ] Instalación completada; SmartScreen superado.
- [ ] NominaMX abierta desde el menú Inicio.
- [ ] Empresa `XAXX010101000`, régimen `601` creada.
- [ ] Trabajador José Pérez Núñez dado de alta con los datos indicados.
- [ ] Periodo quincenal de enero 2026 calculado.
- [ ] Botón **Descargar PDF** descargó `recibo_<id>.pdf`.
- [ ] PDF abierto en Edge con acentos correctos.
- [ ] `nomina.db` verificado en `%APPDATA%\com.nomina.mx\`.
- [ ] Cinco capturas guardadas en `docs/evidencia/verificacion_visual/windows/`.
- [ ] `docs/smoke_test.md` actualizado con resultados reales.

> **Nota:** Parallels Pro ($99 USD/año) permitiría automatizar con
> `prlctl exec/capture`. No requerido para v0.9; considerar para v1.0 si se
> hacen releases recurrentes.

## Errores comunes

| Síntoma | Cómo resolverlo |
|---------|-----------------|
| "Windows protegió su PC" al instalar | **Más información → Ejecutar de todas formas**. Es la advertencia por falta de firma de código. |
| No aparece `\\Mac\Home\Desktop\` | En Parallels, habilitar "Compartir carpeta de inicio de Mac"; también se puede usar `\\Mac\Home\` y navegar a `Desktop`. |
| `Win+Shift+S` no captura | Abrir **Recortes** desde el menú Inicio y usar **Nueva captura**. |
| PDF con `JosÃ©` o acentos rotos | Adjuntar captura y reportar como **bloqueante específico** de encoding en `@react-pdf/renderer` bajo Windows. |
| No se crea `nomina.db` | Verificar permisos de `%APPDATA%\com.nomina.mx\` desde la cuenta de Windows usada; crear la carpeta si falta y volver a abrir la app. |
| No hay run verde o falla la descarga | Ejecutar `gh auth status` en el host; el artefacto se llama `NominaMX-windows-latest`. |

**Resultado esperado:** Windows queda "✅ Verificado". Solo entonces se puede
declarar `v0.9.0` e iniciar el piloto con contadores. Hasta ese momento: tag no
creado y piloto no iniciado.
