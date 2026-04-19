# ── Stage 1: builder ────────────────────────────────────────────────────────
FROM node:20-alpine AS builder

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY . .

# Build with a placeholder; the real address is injected at runtime by entrypoint.sh
ARG VITE_RELAY_API_BASE=http://localhost:8080
ARG VITE_MONAD_RPC_URL=https://testnet-rpc.monad.xyz
ARG VITE_INTENT_DOMAIN_NAME=IntentSwap Relay
ARG VITE_INTENT_DOMAIN_VERSION=1
ARG VITE_INTENT_CHAIN_ID=10143
# Placeholder replaced at container start-up
ARG VITE_INTENT_VERIFYING_CONTRACT=__CONTRACT_PLACEHOLDER__

ENV VITE_RELAY_API_BASE=$VITE_RELAY_API_BASE \
    VITE_MONAD_RPC_URL=$VITE_MONAD_RPC_URL \
    VITE_INTENT_DOMAIN_NAME=$VITE_INTENT_DOMAIN_NAME \
    VITE_INTENT_DOMAIN_VERSION=$VITE_INTENT_DOMAIN_VERSION \
    VITE_INTENT_CHAIN_ID=$VITE_INTENT_CHAIN_ID \
    VITE_INTENT_VERIFYING_CONTRACT=$VITE_INTENT_VERIFYING_CONTRACT

RUN npm run build

# ── Stage 2: runtime ─────────────────────────────────────────────────────────
FROM nginx:1.27-alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY docker-entrypoint.sh /docker-entrypoint.sh

RUN chmod +x /docker-entrypoint.sh

EXPOSE 80

ENTRYPOINT ["/docker-entrypoint.sh"]
