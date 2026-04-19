import { useCallback, useEffect, useMemo, useState } from "react";

import { StoredIntent, listIntents } from "../lib/api";

interface IntentListProps {
  refreshSeed: number;
  defaultMaker?: string;
}

export default function IntentList({ refreshSeed, defaultMaker }: IntentListProps) {
  const [makerFilter, setMakerFilter] = useState("");
  const [data, setData] = useState<StoredIntent[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadIntents = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const intents = await listIntents(makerFilter.trim() || undefined);
      setData(intents);
    } catch (requestError) {
      setError(requestError instanceof Error ? requestError.message : "No se pudieron cargar intents");
    } finally {
      setLoading(false);
    }
  }, [makerFilter]);

  useEffect(() => {
    if (defaultMaker) {
      setMakerFilter(defaultMaker);
    }
  }, [defaultMaker]);

  useEffect(() => {
    loadIntents();
    const timer = window.setInterval(loadIntents, 5000);
    return () => window.clearInterval(timer);
  }, [loadIntents, refreshSeed]);

  const rows = useMemo(() => {
    return data.map((intent) => (
      <article key={intent.intentHash} className="intent-card">
        <header>
          <h3>{intent.intentHash.slice(0, 18)}...</h3>
          <span className={`status status-${intent.status.toLowerCase()}`}>{intent.status}</span>
        </header>
        <p>
          {intent.intent.amountIn} de {intent.intent.tokenIn} {"->"} min {intent.intent.minAmountOut} de {intent.intent.tokenOut}
        </p>
        <p>Maker: {intent.intent.maker}</p>
        <p>Deadline: {new Date(intent.intent.deadline * 1000).toLocaleString()}</p>
      </article>
    ));
  }, [data]);

  return (
    <section className="panel">
      <div className="panel-title">
        <h2>Intents activas</h2>
        <div className="controls">
          <input
            placeholder="Filtrar por maker"
            value={makerFilter}
            onChange={(event) => setMakerFilter(event.target.value)}
          />
          <button onClick={loadIntents} disabled={loading}>
            {loading ? "Actualizando..." : "Refrescar"}
          </button>
        </div>
      </div>

      {error ? <div className="toast error">{error}</div> : null}
      <div className="intent-list">{rows.length ? rows : <p>No hay intents registradas.</p>}</div>
    </section>
  );
}
