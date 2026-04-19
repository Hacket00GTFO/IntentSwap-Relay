import { Hex, isHex, padHex, stringToHex } from "viem";

import { IntentPayload } from "./api";

const domainName = import.meta.env.VITE_INTENT_DOMAIN_NAME ?? "IntentSwap Relay";
const domainVersion = import.meta.env.VITE_INTENT_DOMAIN_VERSION ?? "1";
const chainId = Number(import.meta.env.VITE_INTENT_CHAIN_ID ?? "10143");
const verifyingContract =
  (import.meta.env.VITE_INTENT_VERIFYING_CONTRACT as Hex | undefined) ??
  "0x0000000000000000000000000000000000000000";

const ZERO_ADDRESS = "0x0000000000000000000000000000000000000000";

export const intentEip712Types = {
  Intent: [
    { name: "maker", type: "address" },
    { name: "tokenIn", type: "address" },
    { name: "tokenOut", type: "address" },
    { name: "amountIn", type: "uint256" },
    { name: "minAmountOut", type: "uint256" },
    { name: "receiver", type: "address" },
    { name: "deadline", type: "uint256" },
    { name: "nonce", type: "uint256" },
    { name: "salt", type: "bytes32" },
    { name: "maxRelayerFeeBps", type: "uint16" },
    { name: "allowedRelayer", type: "address" },
    { name: "referralCode", type: "bytes32" },
    { name: "partialFillAllowed", type: "bool" }
  ]
} as const;

export const intentDomain = {
  name: domainName,
  version: domainVersion,
  chainId,
  verifyingContract
} as const;

export function normalizeBytes32(input: string): Hex {
  const trimmed = input.trim();

  if (!trimmed) {
    return "0x0000000000000000000000000000000000000000000000000000000000000000";
  }

  if (isHex(trimmed)) {
    return padHex(trimmed, { size: 32 });
  }

  return padHex(stringToHex(trimmed), { size: 32 });
}

export function buildTypedMessage(intent: IntentPayload) {
  return {
    maker: intent.maker as Hex,
    tokenIn: intent.tokenIn as Hex,
    tokenOut: intent.tokenOut as Hex,
    amountIn: BigInt(intent.amountIn),
    minAmountOut: BigInt(intent.minAmountOut),
    receiver: intent.receiver as Hex,
    deadline: BigInt(intent.deadline),
    nonce: BigInt(intent.nonce),
    salt: normalizeBytes32(intent.salt),
    maxRelayerFeeBps: intent.maxRelayerFeeBps,
    allowedRelayer: ((intent.allowedRelayer?.trim() || ZERO_ADDRESS) as Hex),
    referralCode: normalizeBytes32(intent.referralCode?.trim() || ""),
    partialFillAllowed: intent.partialFillAllowed
  };
}
