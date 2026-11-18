# NominaMX

Aplicacion de escritorio para administracion de nomina mexicana (Tauri 2 + React).

## Requisitos
- Node.js 20+, pnpm
- Rust toolchain (cargo), Tauri CLI 2
- (macOS) Xcode command line tools; (Windows) WebView2 + MSVC

## Instalacion
```bash
pnpm install
cd src-tauri && cargo build && cd ..
```

## Desarrollo
```bash
pnpm tauri:dev
```

## Tests
```bash
# Frontend
pnpm test
# Backend
cd src-tauri && cargo test
```

## Build produccion
```bash
pnpm tauri:build
```
Genera `.dmg` (macOS) y `.msi` (Windows) en `src-tauri/target/release/bundle`.

## Notas
- Parametros fiscales en BD (tablas_fiscales), editables desde Configuracion.
- CFDI: integracion PAC configurable (sandbox). XML generado en comando generar_cfdi.

## Limitaciones conocidas de esta entrega

- La app genera CFDI 4.0 Nómina 1.2 con estructura válida (XSD SAT local), pero NO lo timbra.
- El Sello y Certificado del XML son de ejemplo, no criptográficos.
- No hay integración con PAC en esta entrega (sin CSD ni contrato).
- Horas extra implementadas en Fase 9 / Bloque B (LFT 66/68, LISR 93 I, CFDI 019).
- Módulos pendientes v1.1: prima dominical, festivos, PTU, ajuste anual, importación CSV, backup automático, log de auditoría, multi-empresa, recálculo retroactivo.
- Ver docs/smoke_test.md y docs/matriz_cumplimiento.md para detalle.

## Estado de fases
- Fase 1: COMPLETA
- Fase 2: COMPLETA
- Fase 3: COMPLETA
- Fase 4: COMPLETA
- Fase 5: COMPLETA
- Fase 6: COMPLETA
- Fase 7: COMPLETA
- Fase 8: PARCIAL (firma pendiente de certificados + checklist de puntos 1-7 pendiente; build .dmg OK, tests verdes)
- Fase 9: COMPLETA (A1-A9). CI en GitHub Actions con matrix macOS/Windows en verde (run 44016157633). Pendiente no bloqueante para release: verificación visual del PDF con acentos en Edge y de la ruta SQLite en Windows.
- GitHub: https://github.com/CarlosPrado78/nomina-mx