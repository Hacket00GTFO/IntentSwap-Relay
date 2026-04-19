#!/bin/sh
set -e

SHARED_DIR="${SHARED_DIR:-/shared}"
CONTRACT_FILE="$SHARED_DIR/engine_address.txt"
HTML_DIR="/usr/share/nginx/html"
PLACEHOLDER="__CONTRACT_PLACEHOLDER__"

# Wait up to 120 s for the engine address
WAIT=0
MAX_WAIT=120
until [ -f "$CONTRACT_FILE" ]; do
  if [ $WAIT -ge $MAX_WAIT ]; then
    echo "[frontend] ERROR: timed out waiting for engine_address.txt" >&2
    exit 1
  fi
  echo "[frontend] Waiting for engine address ($WAIT s)..."
  sleep 2
  WAIT=$((WAIT + 2))
done

ENGINE_ADDRESS=$(cat "$CONTRACT_FILE" | tr -d '[:space:]')

if ! echo "$ENGINE_ADDRESS" | grep -qE '^0x[0-9a-fA-F]{40}$'; then
  echo "[frontend] ERROR: invalid engine address '$ENGINE_ADDRESS'" >&2
  exit 1
fi

echo "[frontend] Injecting ENGINE_ADDRESS=$ENGINE_ADDRESS into JS bundle..."

# Replace placeholder in all compiled JS files
find "$HTML_DIR" -name "*.js" | while read -r file; do
  sed -i "s/$PLACEHOLDER/$ENGINE_ADDRESS/g" "$file"
done

echo "[frontend] Placeholder replaced. Starting nginx..."
exec nginx -g "daemon off;"
