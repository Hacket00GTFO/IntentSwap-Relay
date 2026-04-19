interface ImportMetaEnv {
  readonly VITE_RELAY_API_BASE?: string;
  readonly VITE_MONAD_RPC_URL?: string;
  readonly VITE_INTENT_DOMAIN_NAME?: string;
  readonly VITE_INTENT_DOMAIN_VERSION?: string;
  readonly VITE_INTENT_CHAIN_ID?: string;
  readonly VITE_INTENT_VERIFYING_CONTRACT?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
