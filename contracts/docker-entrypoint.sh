#!/bin/sh
set -e

SHARED_DIR="${SHARED_DIR:-/shared}"
OUTPUT_FILE="$SHARED_DIR/engine_address.txt"

mkdir -p "$SHARED_DIR"

# If a previous deploy left a valid address, skip re-deploying
if [ -f "$OUTPUT_FILE" ]; then
  EXISTING=$(cat "$OUTPUT_FILE" | tr -d '[:space:]')
  if echo "$EXISTING" | grep -qE '^0x[0-9a-fA-F]{40}$'; then
    echo "[deploy] Engine address already present: $EXISTING — skipping deploy."
    exit 0
  fi
fi

if [ -z "$DEPLOYER_PRIVATE_KEY" ]; then
  echo "[deploy] ERROR: DEPLOYER_PRIVATE_KEY is not set." >&2
  exit 1
fi

if [ -z "$DEPLOYER_ADDRESS" ]; then
  echo "[deploy] ERROR: DEPLOYER_ADDRESS is not set." >&2
  exit 1
fi

RPC_URL="${MONAD_RPC_URL:-https://testnet-rpc.monad.xyz}"

echo "[deploy] Running forge script against $RPC_URL..."

FORGE_OUTPUT=$(forge script script/Deploy.s.sol:Deploy \
  --rpc-url "$RPC_URL" \
  --private-key "$DEPLOYER_PRIVATE_KEY" \
  --broadcast \
  --slow \
  2>&1)

echo "$FORGE_OUTPUT"

ENGINE_ADDRESS=$(echo "$FORGE_OUTPUT" | grep -oP '(?<=ENGINE_ADDRESS: )0x[0-9a-fA-F]{40}')

if [ -z "$ENGINE_ADDRESS" ]; then
  echo "[deploy] ERROR: Could not parse ENGINE_ADDRESS from forge output." >&2
  exit 1
fi

echo "[deploy] ExecutionEngine deployed at: $ENGINE_ADDRESS"
echo -n "$ENGINE_ADDRESS" > "$OUTPUT_FILE"

echo "[deploy] Address written to $OUTPUT_FILE"
