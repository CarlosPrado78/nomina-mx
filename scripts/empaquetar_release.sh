#!/usr/bin/env bash
# Descarga los bundles .dmg y .msi del último CI verde y los coloca en dist/.
# Uso: scripts/empaquetar_release.sh
# Requiere gh autenticado (o exportar GH_BIN / GH_TOKEN).

set -uo pipefail

REPO='CarlosPrado78/nomina-mx'
GH_BIN="${GH_BIN:-gh}"
SALIDA='dist'
FALLO=0

mkdir -p "$SALIDA"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

command -v "$GH_BIN" >/dev/null 2>&1 || {
  echo "ERROR: no se encontró gh. Exporta GH_BIN o GH_TOKEN." >&2
  exit 1
}

RUN_ID="$("$GH_BIN" run list --repo "$REPO" --branch main --limit 30 \
  --json databaseId,conclusion,event \
  --jq '[.[] | select(.conclusion == "success" and .event == "push")][0].databaseId' 2>/dev/null)"

if [ -z "$RUN_ID" ]; then
  echo "ERROR: no hay un run verde reciente en main." >&2
  exit 1
fi
echo "Usando run $RUN_ID"

descargar() {
  nombre="$1"
  destino_tmp="$TMP_DIR/$nombre"
  mkdir -p "$destino_tmp"
  if ! "$GH_BIN" run download "$RUN_ID" --repo "$REPO" \
      --name "$nombre" --dir "$destino_tmp"; then
    echo "ERROR: no se pudo descargar $nombre" >&2
    FALLO=1
    return
  fi
  encontrado=0
  while IFS= read -r archivo; do
    cp "$archivo" "$SALIDA/" || FALLO=1
    echo "  -> $SALIDA/$(basename "$archivo")"
    encontrado=1
  done < <(find "$destino_tmp" -type f \( -name '*.dmg' -o -name '*.msi' \) -print)
  if [ "$encontrado" -eq 0 ]; then
    echo "ERROR: $nombre no contiene .dmg ni .msi" >&2
    FALLO=1
  fi
}

descargar 'NominaMX-macos-arm64'
descargar 'NominaMX-macos-x86_64'
descargar 'NominaMX-windows-latest'

echo ""
echo "Archivos en $SALIDA:"
ls -lh "$SALIDA" | grep -E '\.(dmg|msi)$' || true
exit "$FALLO"
