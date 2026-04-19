import { useEffect, useMemo } from "react";

import { useAccount, useConnect, useDisconnect } from "wagmi";

interface WalletBarProps {
  onAddressChange: (address: string) => void;
}

function shortAddress(address: string): string {
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

export default function WalletBar({ onAddressChange }: WalletBarProps) {
  const { address, isConnected } = useAccount();
  const { connect, connectors, isPending } = useConnect();
  const { disconnect } = useDisconnect();

  const connector = useMemo(() => connectors[0], [connectors]);

  useEffect(() => {
    onAddressChange(address ?? "");
  }, [address, onAddressChange]);

  return (
    <section className="wallet-bar panel">
      <div>
        <h2>Wallet</h2>
        <p>
          {isConnected && address
            ? `Conectada: ${shortAddress(address)}`
            : "Conecta una wallet para firmar intents EIP-712."}
        </p>
      </div>

      {isConnected ? (
        <button type="button" onClick={() => disconnect()}>
          Desconectar
        </button>
      ) : (
        <button
          type="button"
          disabled={!connector || isPending}
          onClick={() => {
            if (connector) {
              connect({ connector });
            }
          }}
        >
          {isPending ? "Conectando..." : "Conectar Wallet"}
        </button>
      )}
    </section>
  );
}
