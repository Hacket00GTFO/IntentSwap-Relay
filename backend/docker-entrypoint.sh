#!/bin/sh
set -e

SHARED_DIR="${SHARED_DIR:-/shared}"
CONTRACT_FILE="$SHARED_DIR/engine_address.txt"

# Wait up to 120 s for the contracts-deploy service to write the address
WAIT=0
MAX_WAIT=120
until [ -f "$CONTRACT_FILE" ]; do
  if [ $WAIT -ge $MAX_WAIT ]; then
    echo "[backend] ERROR: timed out waiting for engine_address.txt" >&2
    exit 1
  fi
  echo "[backend] Waiting for engine address ($WAIT s)..."
  sleep 2
  WAIT=$((WAIT + 2))
done

ENGINE_ADDRESS=$(cat "$CONTRACT_FILE" | tr -d '[:space:]')

if ! echo "$ENGINE_ADDRESS" | grep -qE '^0x[0-9a-fA-F]{40}$'; then
  echo "[backend] ERROR: invalid engine address '$ENGINE_ADDRESS'" >&2
  exit 1
fi

export INTENT_VERIFYING_CONTRACT="$ENGINE_ADDRESS"
echo "[backend] INTENT_VERIFYING_CONTRACT=$INTENT_VERIFYING_CONTRACT"

exec ./intent-relay-backend
