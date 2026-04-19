import { FormEvent, useEffect, useMemo, useState } from "react";
import { isAddress } from "viem";
import { useSignTypedData } from "wagmi";

import { IntentPayload, createIntent } from "../lib/api";
import { buildTypedMessage, intentDomain, intentEip712Types } from "../lib/eip712";

interface IntentFormProps {
  connectedAddress?: string;
  onCreated: () => void;
}

const BPS_CAP = 10_000;
const ZERO_ADDRESS = "0x0000000000000000000000000000000000000000";

function randomBytes32Hex(): string {
  const bytes = new Uint8Array(32);
  crypto.getRandomValues(bytes);
  return `0x${Array.from(bytes, (value) => value.toString(16).padStart(2, "0")).join("")}`;
}

function toUnixTimestamp(value: string): number {
  return Math.floor(new Date(value).getTime() / 1000);
}

export default function IntentForm({ connectedAddress, onCreated }: IntentFormProps) {
  const [maker, setMaker] = useState(connectedAddress || ZERO_ADDRESS);
  const [receiver, setReceiver] = useState(connectedAddress || ZERO_ADDRESS);
  const [tokenIn, setTokenIn] = useState("0x1111111111111111111111111111111111111111");
  const [tokenOut, setTokenOut] = useState("0x2222222222222222222222222222222222222222");
  const [amountIn, setAmountIn] = useState("1000000");
  const [minAmountOut, setMinAmountOut] = useState("995000");
  const [deadline, setDeadline] = useState(() => {
    const date = new Date();
    date.setMinutes(date.getMinutes() + 20);
    return date.toISOString().slice(0, 16);
  });
  const [nonce, setNonce] = useState(0);
  const [salt, setSalt] = useState(() => randomBytes32Hex());
  const [maxRelayerFeeBps, setMaxRelayerFeeBps] = useState(35);
  const [allowedRelayer, setAllowedRelayer] = useState("");
  const [referralCode, setReferralCode] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const { signTypedDataAsync, isPending: isSigning } = useSignTypedData();

  const deadlineUnix = useMemo(() => toUnixTimestamp(deadline), [deadline]);

  useEffect(() => {
    if (connectedAddress) {
      setMaker(connectedAddress);
      setReceiver(connectedAddress);
    }
  }, [connectedAddress]);

  const effectiveLoading = loading || isSigning;

  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    setError(null);
    setSuccess(null);

    if (!connectedAddress) {
      setError("Conecta una wallet para firmar la intent.");
      return;
    }

    if (Number.isNaN(deadlineUnix) || deadlineUnix <= Math.floor(Date.now() / 1000)) {
      setError("El deadline debe estar en el futuro.");
      return;
    }

    if (maxRelayerFeeBps < 0 || maxRelayerFeeBps > BPS_CAP) {
      setError("maxRelayerFeeBps debe estar entre 0 y 10000.");
      return;
    }

    if (maker.toLowerCase() !== connectedAddress.toLowerCase()) {
      setError("El maker debe coincidir con la wallet conectada.");
      return;
    }

    const addressFields = [
      { field: "Maker", value: maker },
      { field: "Receiver", value: receiver },
      { field: "Token In", value: tokenIn },
      { field: "Token Out", value: tokenOut }
    ];

    for (const entry of addressFields) {
      if (!isAddress(entry.value)) {
        setError(`${entry.field} debe ser una direccion EVM valida.`);
        return;
      }
    }

    if (allowedRelayer.trim() && !isAddress(allowedRelayer.trim())) {
      setError("Allowed Relayer debe ser una direccion valida si se especifica.");
      return;
    }

    const payload: IntentPayload = {
      maker,
      tokenIn,
      tokenOut,
      amountIn,
      minAmountOut,
      receiver,
      deadline: deadlineUnix,
      nonce,
      salt,
      maxRelayerFeeBps,
      allowedRelayer: allowedRelayer.trim() || undefined,
      referralCode: referralCode.trim() || undefined,
      partialFillAllowed: false
    };

    setLoading(true);
    try {
      const signature = await signTypedDataAsync({
        domain: intentDomain,
        types: intentEip712Types,
        primaryType: "Intent",
        message: buildTypedMessage(payload)
      });

      const created = await createIntent(payload, signature);
      setSuccess(`Intent creada: ${created.intentHash}`);
      setNonce((value) => value + 1);
      setSalt(randomBytes32Hex());
      onCreated();
    } catch (requestError) {
      setError(requestError instanceof Error ? requestError.message : "Error inesperado");
    } finally {
      setLoading(false);
    }
  }

  return (
    <form className="panel form" onSubmit={onSubmit}>
      <h2>Crear Intent</h2>
      <p>Vende un token con salida minima garantizada y deadline fijo.</p>

      <div className="grid">
        <label>
          Maker
          <input value={maker} onChange={(event) => setMaker(event.target.value)} required readOnly />
        </label>
        <label>
          Receiver
          <input value={receiver} onChange={(event) => setReceiver(event.target.value)} required />
        </label>
        <label>
          Token In
          <input value={tokenIn} onChange={(event) => setTokenIn(event.target.value)} required />
        </label>
        <label>
          Token Out
          <input value={tokenOut} onChange={(event) => setTokenOut(event.target.value)} required />
        </label>
        <label>
          Amount In
          <input value={amountIn} onChange={(event) => setAmountIn(event.target.value)} required />
        </label>
        <label>
          Min Amount Out
          <input value={minAmountOut} onChange={(event) => setMinAmountOut(event.target.value)} required />
        </label>
        <label>
          Deadline
          <input type="datetime-local" value={deadline} onChange={(event) => setDeadline(event.target.value)} required />
        </label>
        <label>
          Nonce
          <input
            type="number"
            value={nonce}
            min={0}
            onChange={(event) => setNonce(Number(event.target.value))}
            required
          />
        </label>
        <label>
          Salt
          <input value={salt} onChange={(event) => setSalt(event.target.value)} required />
        </label>
        <label>
          Max Relayer Fee (bps)
          <input
            type="number"
            value={maxRelayerFeeBps}
            min={0}
            max={BPS_CAP}
            onChange={(event) => setMaxRelayerFeeBps(Number(event.target.value))}
            required
          />
        </label>
        <label>
          Allowed Relayer (opcional)
          <input value={allowedRelayer} onChange={(event) => setAllowedRelayer(event.target.value)} />
        </label>
        <label>
          Referral Code (opcional)
          <input value={referralCode} onChange={(event) => setReferralCode(event.target.value)} />
        </label>
      </div>

      <button type="submit" disabled={effectiveLoading || !connectedAddress}>
        {effectiveLoading ? "Firmando y enviando..." : "Firmar y crear intent"}
      </button>

      {success ? <div className="toast success">{success}</div> : null}
      {error ? <div className="toast error">{error}</div> : null}
    </form>
  );
}
