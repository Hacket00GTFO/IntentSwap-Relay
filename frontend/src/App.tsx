import { useState } from "react";

import IntentForm from "./components/IntentForm";
import IntentList from "./components/IntentList";
import WalletBar from "./components/WalletBar";

export default function App() {
  const [refreshSeed, setRefreshSeed] = useState(0);
  const [walletAddress, setWalletAddress] = useState("");

  return (
    <main className="app-shell">
      <div className="ambient" aria-hidden="true" />
      <header className="hero panel">
        <p className="eyebrow">IntentSwap Relay</p>
        <h1>Firma una intencion, deja que los relayers compitan por el mejor fill.</h1>
        <p>
          MVP inicial con relay off-chain, broadcast WebSocket y estados de ejecucion para Monad.
        </p>
      </header>

      <WalletBar onAddressChange={setWalletAddress} />

      <section className="layout">
        <IntentForm
          connectedAddress={walletAddress}
          onCreated={() => setRefreshSeed((value) => value + 1)}
        />
        <IntentList refreshSeed={refreshSeed} defaultMaker={walletAddress} />
      </section>
    </main>
  );
}
