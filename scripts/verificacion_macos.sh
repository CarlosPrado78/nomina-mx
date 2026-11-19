#!/usr/bin/env bash
# Verificación post-instalación de NominaMX en un usuario nuevo de macOS.
# Ejecutar DENTRO de la sesión del usuario VerificacionNominaMX.
# Descarga el bundle de la arquitectura de la Mac donde se ejecuta (arm64 o
# x86_64). No automatiza el alta de empresa/trabajador/recibo: eso se hace con
# clicks humanos y se documenta con capturas manuales.

set -uo pipefail

REPO='CarlosPrado78/nomina-mx'
GH_BIN="${GH_BIN:-gh}"
EVID_DIR='docs/evidencia/verificacion_visual/macos'
REPORTE="$EVID_DIR/reporte.md"
DIST_DIR='dist/macos'
TMP_DIR="$(mktemp -d)"
FALLO=0

# El bundle anterior al commit 71a5b7c crasheaba en el primer arranque con BD
# legacy (checksum de la migración 2). Nunca instalar un run anterior a este.
RUN_MINIMO=44489870148

ARCH="$(uname -m)"
case "$ARCH" in
  arm64)
    ARTIFACT='NominaMX-macos-arm64'
    ESPERADO='arm64'
    ;;
  x86_64)
    ARTIFACT='NominaMX-macos-x86_64'
    ESPERADO='x86_64'
    ;;
  *)
    echo "Arquitectura no soportada: $ARCH" >&2
    exit 1
    ;;
esac

mkdir -p "$EVID_DIR" "$DIST_DIR"
: > "$REPORTE"

echo "# Reporte de verificación macOS (usuario nuevo)" >> "$REPORTE"
echo "" >> "$REPORTE"
echo "Fecha: $(date '+%Y-%m-%d %H:%M:%S')" >> "$REPORTE"
echo "Usuario: $(whoami)" >> "$REPORTE"
echo "Arquitectura: $ARCH (artefacto $ARTIFACT)" >> "$REPORTE"
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

registrar 'Detectar arquitectura' '✅' "$ARCH → $ARTIFACT"

command -v "$GH_BIN" >/dev/null 2>&1 || {
  fallar "No se encontró gh. Exporta GH_BIN=/ruta/a/gh o GH_TOKEN antes de ejecutar."
  registrar 'Preparación' '❌' 'gh no disponible'
  exit 1
}

RUN_ID="$("$GH_BIN" run list --repo "$REPO" --workflow=ci.yml --status=success --limit=1 \
  --json databaseId --jq '.[0].databaseId' 2>/dev/null)"

if [ -z "$RUN_ID" ]; then
  fallar 'No se encontró un run verde de ci.yml.'
  registrar 'Descargar artefacto' '❌' 'sin run verde de ci.yml'
  exit 1
fi

if [ "$RUN_ID" -lt "$RUN_MINIMO" ] 2>/dev/null; then
  fallar "El run $RUN_ID es anterior al fix de migración (mínimo $RUN_MINIMO). No se instalará."
  registrar 'Descargar artefacto' '❌' "run $RUN_ID anterior al fix"
  exit 1
fi
registrar 'Descargar artefacto' "✅ run $RUN_ID" "último run verde de ci.yml ($ARTIFACT)"

# Evitar que un .dmg descargado antes para otra arquitectura sea elegido por find.
rm -f "$DIST_DIR"/*.dmg 2>/dev/null || true

if ! "$GH_BIN" run download "$RUN_ID" --repo "$REPO" \
    --name "$ARTIFACT" --dir "$DIST_DIR"; then
  fallar "No se pudo descargar $ARTIFACT del run $RUN_ID. Verifica que el run incluya los artefactos separados por arquitectura."
  registrar 'Descargar artefacto' '❌' "gh run download $ARTIFACT falló"
  exit 1
fi

DMG="$(find "$DIST_DIR" -type f -name '*.dmg' -print -quit)"
if [ -z "$DMG" ]; then
  fallar "El artefacto $ARTIFACT no contiene un .dmg."
  registrar 'Obtener .dmg' '❌' 'no se encontró archivo .dmg'
  exit 1
fi
registrar 'Obtener .dmg' '✅' "$(basename "$DMG")"

# Comprobación pedida sobre el contenedor. Un .dmg es una imagen (Apple Disk
# Image), no un Mach-O: la validación real de arquitectura se hace más abajo,
# sobre el ejecutable dentro del bundle ya montado.
if file "$DMG" | grep -q "Mach-O 64-bit"; then
  registrar 'Formato del .dmg' '✅' "$(file -b "$DMG")"
else
  echo "ADVERTENCIA: .dmg no es Mach-O"
  registrar 'Formato del .dmg' 'ℹ️' 'contenedor de imagen; se valida el binario al montarlo'
fi

MONTURA="$TMP_DIR/montura"
mkdir -p "$MONTURA"
if ! hdiutil attach "$DMG" -mountpoint "$MONTURA" -nobrowse -quiet; then
  fallar 'No se pudo montar el .dmg.'
  registrar 'Montar .dmg' '❌' 'hdiutil attach falló'
  exit 1
fi
registrar 'Montar .dmg' '✅' 'hdiutil attach'

BIN="$MONTURA/NominaMX.app/Contents/MacOS/nomina-mx"
if [ -f "$BIN" ]; then
  INFO_BIN="$(file -b "$BIN")"
  if printf '%s' "$INFO_BIN" | grep -q "$ESPERADO"; then
    registrar 'Arquitectura del binario' '✅' "$INFO_BIN"
  else
    fallar "El binario del .dmg no es $ESPERADO: $INFO_BIN"
    registrar 'Arquitectura del binario' '❌' "esperado $ESPERADO; obtenido $INFO_BIN"
  fi
else
  fallar 'No se encontró el ejecutable dentro del .dmg.'
  registrar 'Arquitectura del binario' '❌' 'falta Contents/MacOS/nomina-mx'
fi

if [ -d /Applications/NominaMX.app ]; then
  cp -R /Applications/NominaMX.app "$TMP_DIR/NominaMX.app.anterior" 2>/dev/null || true
fi
# cp -R sobre un .app existente fusiona directorios y deja binarios viejos:
# hay que quitar el bundle anterior para que el reemplazo sea real.
if rm -rf /Applications/NominaMX.app 2>/dev/null && \
   cp -R "$MONTURA/NominaMX.app" /Applications/ 2>/dev/null && \
   [ -x /Applications/NominaMX.app/Contents/MacOS/nomina-mx ]; then
  APP='/Applications/NominaMX.app'
  registrar 'Instalar en /Applications' '✅' "$APP (anterior reemplazado; respaldo en $TMP_DIR)"
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
