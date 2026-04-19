export type IntentStatus = "PENDING" | "EXECUTED" | "EXPIRED" | "CANCELLED";

export interface IntentPayload {
  maker: string;
  tokenIn: string;
  tokenOut: string;
  amountIn: string;
  minAmountOut: string;
  receiver: string;
  deadline: number;
  nonce: number;
  salt: string;
  maxRelayerFeeBps: number;
  allowedRelayer?: string;
  referralCode?: string;
  partialFillAllowed: boolean;
}

export interface StoredIntent {
  intentHash: string;
  status: IntentStatus;
  createdAt: string;
  executedAt?: string;
  executedBy?: string;
  finalAmountOut?: string;
  intent: IntentPayload;
}

export interface IntentCreateResponse {
  intentHash: string;
  status: IntentStatus;
  createdAt: string;
}

const API_BASE = import.meta.env.VITE_RELAY_API_BASE ?? "http://localhost:8080";

async function safeJson<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const payload = await response.json().catch(() => ({ error: "unexpected error" }));
    throw new Error(payload.error ?? "request failed");
  }

  return response.json() as Promise<T>;
}

export async function createIntent(intent: IntentPayload, signature: string): Promise<IntentCreateResponse> {
  const response = await fetch(`${API_BASE}/intents`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ intent, signature })
  });

  return safeJson<IntentCreateResponse>(response);
}

export async function listIntents(maker?: string): Promise<StoredIntent[]> {
  const query = maker ? `?maker=${encodeURIComponent(maker)}` : "";
  const response = await fetch(`${API_BASE}/intents${query}`);
  return safeJson<StoredIntent[]>(response);
}

export async function getHealth(): Promise<{ status: string; service: string }> {
  const response = await fetch(`${API_BASE}/health`);
  return safeJson<{ status: string; service: string }>(response);
}
