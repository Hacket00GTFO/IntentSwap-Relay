import { QueryClient } from "@tanstack/react-query";
import { createConfig, http } from "wagmi";
import { injected } from "wagmi/connectors";
import { defineChain } from "viem";

const chainId = Number(import.meta.env.VITE_INTENT_CHAIN_ID ?? "10143");
const rpcUrl = import.meta.env.VITE_MONAD_RPC_URL ?? "https://testnet-rpc.monad.xyz";

export const monadChain = defineChain({
  id: chainId,
  name: "Monad",
  nativeCurrency: {
    name: "MON",
    symbol: "MON",
    decimals: 18
  },
  rpcUrls: {
    default: {
      http: [rpcUrl]
    }
  }
});

export const wagmiConfig = createConfig({
  chains: [monadChain],
  connectors: [injected()],
  transports: {
    [monadChain.id]: http()
  }
});

export const queryClient = new QueryClient();
