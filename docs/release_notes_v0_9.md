# NominaMX v0.9 — Notas de versión (borrador)

> **Pendiente: verificación visual macOS + Windows antes de publicar.** Este
> documento es un borrador. No se ha creado el tag `v0.9.0` ni la GitHub
> Release. No se inicia el piloto con contadores hasta cerrar la verificación
> en macOS y Windows.

## Novedades

- **Cálculo de nómina mexicana** con tablas fiscales 2026: UMA histórica por
  fecha de pago (enero $113.14; febrero en adelante $117.31), subsidio al
  empleo (enero $536.21; febrero en adelante $535.65) e ISR vigente.
- **IMSS 2026**: cuotas obrero/patronales, cesantía, vejez, CEAV por excedente
  de 3 UMA, prima de riesgo y cuota fija; 177 pruebas automatizadas en verde.
- **INFONAVIT**, **SAR**, **ISN por entidad federativa** y factor de integración
  SDI.
- **Recálculo retroactivo** (C5): corrige SBC y recibos usando los parámetros
  vigentes a la fecha de pago original, con auditoría de UMA y subsidio. Motivo
  obligatorio; respeta CFDI ya timbrado.
- **Finiquitos y liquidaciones**, aguinaldo proporcional, prima vacacional y
  vacaciones por antigüedad.
- **Importación masiva de trabajadores** desde CSV con validación fila por fila.
- **Respaldos** de la base SQLite con `VACUUM INTO`, restauración validada y
  copia previa automática (comandos disponibles via IPC; botón visible en v1.0).
- **Multiempresa** y selector de empresa en el encabezado.
- **PDF de recibo** con acentos (`José Pérez Núñez`) generado con
  `@react-pdf/renderer` y **botón Descargar PDF** enlazado en cada renglón de la
  tabla de resultados de nómina.
- **Migraciones propias sin checksum frágil**: el arranque ya no depende de
  `tauri_plugin_sql`. Las BDs antiguas conservan sus datos y reciben la
  corrección fiscal 2026 como migración nueva (versión 6).

## Limitaciones conocidas

- **Sin firma de código:** macOS muestra Gatekeeper y Windows muestra
  SmartScreen la primera vez; hay que usar "Abrir de todas formas" / "Ejecutar
  de todas formas". La firma está planeada para después del piloto.
- **Timbrado real con PAC y CSD no implementado.** Los comprobantes son de
  muestra y no tienen validez fiscal.
- La **prestación económica por incapacidad** (subsidio IMSS/pago patronal) no
  tiene pantalla ni comando; solo se valida que las cuotas no cambien.
- El **CFDI de ajuste** (`TipoDeComprobante=E`) para recálculos está pendiente
  para v1.1.
- Los recibos con CFDI timbrado **no se modifican** en el recálculo; solo se
  reporta la diferencia.
- Si el recibo original tenía horas extra, el recálculo las reconstruye como
  jornada diurna y una semana.
- La verificación end-to-end en usuario nuevo de macOS y en Windows está
  pendiente; ver `docs/smoke_test.md`.

## Requisitos

- macOS 11 o superior (bundle `aarch64`).
- Windows 10 o superior (bundle `x64`).
- No se requiere conexión a Internet para calcular.

## Instalación y avisos del sistema

- **macOS:** abrir el `.dmg`, arrastrar `NominaMX.app` a Aplicaciones. Si
  Gatekeeper bloquea la apertura por falta de firma, hacer **clic derecho →
  Abrir** o ir a **Preferencias del Sistema → Seguridad y Privacidad → Abrir de
  todas formas**. Solo se requiere la primera vez.
- **Windows:** ejecutar el `.msi`. Si SmartScreen muestra una advertencia,
  hacer clic en **Más información → Ejecutar de todas formas**. Solo se
  requiere la primera vez.

Los bundles se obtienen con `scripts/empaquetar_release.sh`, que descarga los
artefactos `.dmg` y `.msi` del último CI verde a `dist/`.

### Actualización desde builds anteriores

Si ya tenía instalado un bundle que cerraba la app con
`migration 2 was previously applied but has been modified`, instale el bundle
corregido. Como alternativa temporal, antes de abrir la app puede renombrar la
base:

```bash
mv "$HOME/Library/Application Support/com.nomina.mx/nomina.db" \
   "$HOME/Library/Application Support/com.nomina.mx/nomina.db.antiguo"
```

Esto no borra datos de negocio: en la BD afectada no había empresas,
trabajadores, periodos ni recibos. La app corregida crea una BD nueva con el
seed fiscal correcto y también puede migrar la BD antigua sin perder registros.

## Validación y evidencia

- 177 pruebas automatizadas en verde (+1 ignorada), CI en
  `macos-latest` y `windows-latest`. Incluye `src-tauri/tests/migrations.rs`:
  BD nueva, BD legacy de `tauri_plugin_sql` y reintento idempotente.
- El release corregido se compiló localmente y se verificó con una copia de la
  BD legacy: la app quedó abierta, aplicó la migración 6 y no duplicó datos en
  un segundo arranque. Falta repetir la verificación visual en usuario nuevo.
- Casos manuales para el contador: `docs/casos_validacion_contable.md`.
- Manual de uso: `docs/manual_usuario.md`.
- Plan de piloto y feedback: `docs/feedback_piloto.md`.
- Verificación visual: `docs/smoke_test.md`.

## Reporte de bugs

Incluir sistema operativo y versión, pasos para reproducir y captura de
pantalla. No enviar CURP, RFC ni salarios reales en el reporte.
