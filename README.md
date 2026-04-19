# IntentSwap Relay

IntentSwap Relay es un protocolo de intents donde el maker firma una intencion simple ("vender X y recibir al menos Y antes de T") y una red de relayers compite para ejecutarla.

Este repositorio contiene la primera implementacion MVP del monorepo:

- `contracts`: contratos Solidity base con validacion EIP-712, nonce y fee split.
- `backend`: relay API en Rust (REST + WebSocket) con modelo whitelist de relayers.
- `frontend`: UI React para crear intents y monitorear estado.
- `e2e`: espacio reservado para pruebas de extremo a extremo.

## Estructura

```text
.
|- contracts/
|  |- foundry.toml
|  \- src/
|     |- IntentTypes.sol
|     |- IntentManager.sol
|     |- FeeManager.sol
|     |- ExecutionEngine.sol
|     \- IntentFactory.sol
|- backend/
|  |- Cargo.toml
|  |- migrations/
|  |  \- 0001_init.sql
|  |- .env.example
|  \- src/
|     |- main.rs
|     |- api.rs
|     |- db.rs
|     |- models.rs
|     \- signature.rs
|- frontend/
|  |- package.json
|  |- .env.example
|  \- src/
|     |- App.tsx
|     |- components/
|     |  \- WalletBar.tsx
|     \- lib/
|        |- api.ts
|        |- eip712.ts
|        \- wallet.ts
\- e2e/
```

## MVP Implementado

### Contratos

- EIP-712 domain separator por contrato (`IntentManager`).
- Validacion de firma para EOA y EIP-1271 (account abstraction ready).
- Nonce por maker y proteccion anti replay.
- Cancelacion de intent por maker.
- `ExecutionEngine` con validaciones de `minAmountOut`, `allowedRelayer` y fee cap.
- `FeeManager` con fee protocolaria configurable y preview del split.
- `IntentFactory` para desplegar motores de ejecucion.

### Backend (Rust)

- `POST /intents` crea una intent y la publica por broadcast.
- `GET /intents` lista intents (filtro por maker, paginacion basica).
- `GET /intents/:hash/status` devuelve estado.
- `POST /intents/:hash/cancel` cancela intent pendiente.
- `WS /intents/subscribe` transmite intents y cambios de estado.
- `POST /relayers/propose` registra propuesta de relayer (whitelist).
- `GET /relayers` lista relayers activos.
- `GET /analytics/intent/:hash` expone resumen de analytics por intent.
- Persistencia en PostgreSQL con SQLx (`intents`, `relayers`, `relayer_proposals`).
- Seed inicial de relayers desde `RELAY_ALLOWED_RELAYERS`.
- Verificacion de firma EIP-712 real en `POST /intents` (rechaza firmas invalidas).

### Frontend (React + Vite)

- Formulario para crear intents.
- Conexion de wallet (Wagmi) con boton de conectar/desconectar.
- Firma EIP-712 real antes de enviar `POST /intents`.
- Validaciones de deadline y fee bps.
- Vista de intents con refresh automatico.
- Base visual minimalista (modo claro, alto contraste, sombras profundas).

## Ejecutar Local

### 1) Backend

```powershell
Set-Location .\backend
Copy-Item .env.example .env
docker run --name intent-relay-db -e POSTGRES_PASSWORD=postgres -e POSTGRES_USER=postgres -e POSTGRES_DB=intent_relay -p 5432:5432 -d postgres:16
cargo run
```

Backend por defecto en `http://localhost:8080`.

### 2) Frontend

```powershell
Set-Location .\frontend
Copy-Item .env.example .env
npm install
npm run dev
```

Frontend por defecto en `http://localhost:5173`.

Nota: para firmar intents necesitas una wallet inyectada (por ejemplo MetaMask) y estar en la chain configurada por `VITE_INTENT_CHAIN_ID`.

### 3) Contratos (Foundry)

```powershell
Set-Location .\contracts
forge build
```

## Endpoints Base

- `GET /health`
- `POST /intents`
- `GET /intents`
- `GET /intents/{hash}/status`
- `POST /intents/{hash}/cancel`
- `GET /intents/subscribe` (WebSocket)
- `POST /relayers/propose`
- `GET /relayers`
- `GET /analytics/intent/{hash}`

## Siguiente Sprint

1. Integrar ejecucion on-chain real con `ExecutionEngine.executeIntent`.
2. Conectar adaptador Uniswap v4 y ranking de propuestas por output neto.
3. Agregar autenticacion/roles para endpoint de propuestas de relayer.
4. Agregar suite de pruebas (Foundry + Rust + e2e).
5. Incorporar metricas operativas y alertas (latencia, tasa de error, backlog WS).

