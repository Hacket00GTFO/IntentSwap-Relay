use std::{env, str::FromStr};

use anyhow::{anyhow, Context, Result};
use ethers_core::{
    abi::{encode, Token},
    types::{Address, Signature, H256, U256},
    utils::keccak256,
};

use crate::models::IntentPayload;

const EIP712_DOMAIN_TYPE: &str =
    "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)";
const INTENT_TYPE: &str = "Intent(address maker,address tokenIn,address tokenOut,uint256 amountIn,uint256 minAmountOut,address receiver,uint256 deadline,uint256 nonce,bytes32 salt,uint16 maxRelayerFeeBps,address allowedRelayer,bytes32 referralCode,bool partialFillAllowed)";

#[derive(Clone)]
pub struct SignatureConfig {
    pub domain_name: String,
    pub domain_version: String,
    pub chain_id: u64,
    pub verifying_contract: Address,
}

pub fn load_signature_config() -> Result<SignatureConfig> {
    let domain_name =
        env::var("INTENT_DOMAIN_NAME").unwrap_or_else(|_| "IntentSwap Relay".to_string());
    let domain_version = env::var("INTENT_DOMAIN_VERSION").unwrap_or_else(|_| "1".to_string());
    let chain_id = env::var("INTENT_CHAIN_ID")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(1);

    let verifying_contract = env::var("INTENT_VERIFYING_CONTRACT")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| parse_address(&value))
        .transpose()?
        .unwrap_or_else(Address::zero);

    Ok(SignatureConfig {
        domain_name,
        domain_version,
        chain_id,
        verifying_contract,
    })
}

pub fn compute_intent_hash_hex(intent: &IntentPayload) -> Result<String> {
    let struct_hash = compute_intent_struct_hash(intent)?;
    Ok(format!("0x{}", hex::encode(struct_hash)))
}

pub fn verify_eip712_signature(
    config: &SignatureConfig,
    intent: &IntentPayload,
    signature_hex: &str,
) -> Result<bool> {
    let digest = compute_eip712_digest(config, intent)?;
    let signature = Signature::from_str(signature_hex).context("invalid signature encoding")?;
    let recovered = signature
        .recover(H256::from(digest))
        .context("failed recovering signer from signature")?;

    let maker = parse_address(&intent.maker)?;
    Ok(recovered == maker)
}

fn compute_eip712_digest(config: &SignatureConfig, intent: &IntentPayload) -> Result<[u8; 32]> {
    let domain_separator = compute_domain_separator(config);
    let struct_hash = compute_intent_struct_hash(intent)?;

    let mut bytes = Vec::with_capacity(66);
    bytes.push(0x19);
    bytes.push(0x01);
    bytes.extend_from_slice(&domain_separator);
    bytes.extend_from_slice(&struct_hash);

    Ok(keccak256(bytes))
}

fn compute_domain_separator(config: &SignatureConfig) -> [u8; 32] {
    let domain_typehash = keccak256(EIP712_DOMAIN_TYPE.as_bytes());

    let encoded = encode(&[
        Token::FixedBytes(domain_typehash.to_vec()),
        Token::FixedBytes(keccak256(config.domain_name.as_bytes()).to_vec()),
        Token::FixedBytes(keccak256(config.domain_version.as_bytes()).to_vec()),
        Token::Uint(U256::from(config.chain_id)),
        Token::Address(config.verifying_contract),
    ]);

    keccak256(encoded)
}

fn compute_intent_struct_hash(intent: &IntentPayload) -> Result<[u8; 32]> {
    let intent_typehash = keccak256(INTENT_TYPE.as_bytes());

    let maker = parse_address(&intent.maker)?;
    let token_in = parse_address(&intent.token_in)?;
    let token_out = parse_address(&intent.token_out)?;
    let amount_in = parse_uint256(&intent.amount_in)?;
    let min_amount_out = parse_uint256(&intent.min_amount_out)?;
    let receiver = parse_address(&intent.receiver)?;
    let deadline = U256::from(intent.deadline.max(0) as u64);
    let nonce = U256::from(intent.nonce);
    let salt = parse_bytes32(&intent.salt)?;
    let max_relayer_fee_bps = U256::from(intent.max_relayer_fee_bps);

    let allowed_relayer = intent
        .allowed_relayer
        .as_ref()
        .map(|value| parse_address(value))
        .transpose()?
        .unwrap_or_else(Address::zero);

    let referral_code = match intent.referral_code.as_ref() {
        Some(value) if !value.trim().is_empty() => parse_bytes32(value)?,
        _ => [0u8; 32],
    };

    let encoded = encode(&[
        Token::FixedBytes(intent_typehash.to_vec()),
        Token::Address(maker),
        Token::Address(token_in),
        Token::Address(token_out),
        Token::Uint(amount_in),
        Token::Uint(min_amount_out),
        Token::Address(receiver),
        Token::Uint(deadline),
        Token::Uint(nonce),
        Token::FixedBytes(salt.to_vec()),
        Token::Uint(max_relayer_fee_bps),
        Token::Address(allowed_relayer),
        Token::FixedBytes(referral_code.to_vec()),
        Token::Bool(intent.partial_fill_allowed),
    ]);

    Ok(keccak256(encoded))
}

fn parse_uint256(raw: &str) -> Result<U256> {
    if raw.starts_with("0x") || raw.starts_with("0X") {
        U256::from_str(raw).map_err(|_| anyhow!("invalid uint256 value: {raw}"))
    } else {
        U256::from_dec_str(raw).map_err(|_| anyhow!("invalid uint256 decimal value: {raw}"))
    }
}

fn parse_address(raw: &str) -> Result<Address> {
    Address::from_str(raw).map_err(|_| anyhow!("invalid address: {raw}"))
}

fn parse_bytes32(raw: &str) -> Result<[u8; 32]> {
    let value = raw.trim();
    let bytes = if value.starts_with("0x") || value.starts_with("0X") {
        hex::decode(&value[2..]).map_err(|_| anyhow!("invalid hex bytes32 value"))?
    } else {
        hex::decode(value).unwrap_or_else(|_| value.as_bytes().to_vec())
    };

    if bytes.len() > 32 {
        return Err(anyhow!("value too large for bytes32"));
    }

    let mut out = [0u8; 32];
    let start = 32 - bytes.len();
    out[start..].copy_from_slice(&bytes);
    Ok(out)
}
