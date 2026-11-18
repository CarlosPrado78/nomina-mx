#!/usr/bin/env bash
# Verificación post-instalación de NominaMX en un usuario nuevo de macOS.
# Ejecutar DENTRO de la sesión del usuario VerificacionNominaMX.
# No automatiza el alta de empresa/trabajador/recibo: eso se hace con clicks
# humanos y se documenta con capturas manuales.

set -uo pipefail

REPO='CarlosPrado78/nomina-mx'
GH_BIN="${GH_BIN:-gh}"
EVID_DIR='docs/evidencia/verificacion_visual/macos'
REPORTE="$EVID_DIR/reporte.md"
TMP_DIR="$(mktemp -d)"
FALLO=0

mkdir -p "$EVID_DIR"
: > "$REPORTE"

echo "# Reporte de verificación macOS (usuario nuevo)" >> "$REPORTE"
echo "" >> "$REPORTE"
echo "Fecha: $(date '+%Y-%m-%d %H:%M:%S')" >> "$REPORTE"
echo "Usuario: $(whoami)" >> "$REPORTE"
echo "" >> "$REPORTE"
echo "| Paso | Resultado | Captura / nota |" >> "$REPORTE"
echo "|------|-----------|----------------|" >> "$REPORTE"

registrar() {
  echo "| $1 | $2 | $3 |" >> "$REPORTE"
}

fallar() {
  FALLO=1
  echo "ERROR: $*" >&2
}

command -v "$GH_BIN" >/dev/null 2>&1 || {
  fallar "No se encontró gh. Exporta GH_BIN=/ruta/a/gh o GH_TOKEN antes de ejecutar."
  registrar 'Preparación' '❌' 'gh no disponible'
  exit 1
}

RUN_ID="$("$GH_BIN" run list --repo "$REPO" --branch main --limit 30 \
  --json databaseId,conclusion,event \
  --jq '[.[] | select(.conclusion == "success" and .event == "push")][0].databaseId' 2>/dev/null)"

if [ -z "$RUN_ID" ]; then
  fallar 'No se encontró un run verde reciente en main.'
  registrar 'Descargar artefacto' '❌' 'sin run verde'
  exit 1
fi
registrar 'Descargar artefacto' "✅ run $RUN_ID" 'último run verde de main'

if ! "$GH_BIN" run download "$RUN_ID" --repo "$REPO" \
    --name 'NominaMX-macos-latest' --dir "$TMP_DIR"; then
  fallar 'No se pudo descargar NominaMX-macos-latest.'
  registrar 'Descargar artefacto' '❌' 'gh run download falló'
  exit 1
fi

DMG="$(find "$TMP_DIR" -type f -name '*.dmg' -print -quit)"
if [ -z "$DMG" ]; then
  fallar 'El artefacto no contiene un .dmg.'
  registrar 'Obtener .dmg' '❌' 'no se encontró archivo .dmg'
  exit 1
fi
registrar 'Obtener .dmg' '✅' "$(basename "$DMG")"

MONTURA="$TMP_DIR/montura"
mkdir -p "$MONTURA"
if ! hdiutil attach "$DMG" -mountpoint "$MONTURA" -nobrowse -quiet; then
  fallar 'No se pudo montar el .dmg.'
  registrar 'Montar .dmg' '❌' 'hdiutil attach falló'
  exit 1
fi
registrar 'Montar .dmg' '✅' 'hdiutil attach'

if [ -d /Applications/NominaMX.app ]; then
  cp -R /Applications/NominaMX.app "$TMP_DIR/NominaMX.app.anterior" 2>/dev/null || true
fi
if cp -R "$MONTURA/NominaMX.app" /Applications/ 2>/dev/null; then
  APP='/Applications/NominaMX.app'
  registrar 'Instalar en /Applications' '✅' "$APP"
else
  APP="$TMP_DIR/NominaMX.app"
  cp -R "$MONTURA/NominaMX.app" "$APP" || {
    fallar 'No se pudo copiar NominaMX.app.'
    registrar 'Instalar app' '❌' 'cp falló'
    exit 1
  }
  registrar 'Instalar en /Applications' '⚠️' "sin permisos; se usa $APP"
fi

hdiutil detach "$MONTURA" -quiet || true

open -a "$APP" || {
  fallar 'No se pudo abrir la aplicación.'
  registrar 'Abrir app' '❌' 'open -a falló'
  exit 1
}
registrar 'Abrir app' '✅' 'open -a NominaMX.app'
sleep 5

CAPTURA="$EVID_DIR/macos_01_app.png"
if screencapture -x "$CAPTURA" 2>/dev/null; then
  registrar 'Captura app abierta' '✅' 'macos_01_app.png'
else
  fallar 'screencapture sin permisos de grabación de pantalla.'
  registrar 'Captura app abierta' '❌' 'screencapture falló'
fi

DB_DIR="$HOME/Library/Application Support/com.nomina.mx"
if [ -f "$DB_DIR/nomina.db" ]; then
  registrar 'BD creada' '✅' "$DB_DIR/nomina.db"
  ls -la "$DB_DIR" >> "$REPORTE" 2>&1 || true
else
  fallar "No se encontró la BD en $DB_DIR."
  registrar 'BD creada' '❌' "$DB_DIR/nomina.db no existe"
fi

echo "" >> "$REPORTE"
if [ "$FALLO" -eq 0 ]; then
  echo 'Resultado: ✅ Instalación y arranque verificados en usuario nuevo.' >> "$REPORTE"
else
  echo 'Resultado: ❌ Bloqueante específico; revisar las filas marcadas.' >> "$REPORTE"
fi
echo 'Pendiente de ejecutar por una persona: empresa → trabajador → periodo → recibo → PDF → capturas 02 a 05.' >> "$REPORTE"

echo "Reporte generado: $REPORTE"
exit "$FALLO"
