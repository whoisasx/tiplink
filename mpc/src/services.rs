#![allow(deprecated)] // system_instruction: migrate to solana_system_interface in a future cleanup

use aes_gcm::{aead::Aead, Aes256Gcm, Key, Nonce};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use hmac::Hmac;
use sha2::{Digest, Sha256};
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    message::Message,
    program_pack::Pack,
};
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account,
};
use spl_token::state::Mint as SplMint;
use std::str::FromStr;
use uuid::Uuid;

use crate::{config::Config, dto::*, errors::MpcError};

type HmacSha256 = Hmac<Sha256>;

fn derive_seed(master_secret: &str, user_id: &str) -> [u8; 32] {
    use hmac::Mac;
    let mut mac = HmacSha256::new_from_slice(master_secret.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(user_id.as_bytes());
    mac.finalize().into_bytes().into()
}

fn derive_enc_key(master_secret: &str) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(master_secret.as_bytes());
    h.update(b":privkey_enc");
    h.finalize().into()
}

fn derive_nonce(user_id: &str) -> [u8; 12] {
    let hash = Sha256::digest(user_id.as_bytes());
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&hash[..12]);
    nonce
}

fn encrypt_privkey(keypair_bytes: &[u8], master_secret: &str, user_id: &str) -> Result<String, MpcError> {
    use aes_gcm::aead::KeyInit;
    let key         = derive_enc_key(master_secret);
    let nonce_bytes = derive_nonce(user_id);
    let cipher      = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce       = Nonce::from_slice(&nonce_bytes);
    cipher
        .encrypt(nonce, keypair_bytes)
        .map(|ct| hex::encode(ct))
        .map_err(|e| {
            tracing::error!("AES-GCM encrypt failed: {e}");
            MpcError::Internal
        })
}

fn decrypt_privkey(hex_ct: &str, master_secret: &str, user_id: &str) -> Result<Vec<u8>, MpcError> {
    use aes_gcm::aead::KeyInit;
    let ct = hex::decode(hex_ct).map_err(|e| {
        tracing::error!("hex decode failed: {e}");
        MpcError::Internal
    })?;
    let key         = derive_enc_key(master_secret);
    let nonce_bytes = derive_nonce(user_id);
    let cipher      = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let nonce       = Nonce::from_slice(&nonce_bytes);
    cipher.decrypt(nonce, ct.as_slice()).map_err(|e| {
        tracing::error!("AES-GCM decrypt failed: {e}");
        MpcError::Internal
    })
}

fn derive_keypair(master_secret: &str, user_id: &str) -> Keypair {
    let seed = derive_seed(master_secret, user_id);
    Keypair::new_from_array(seed)
}

fn load_keypair_from_enc(hex_ct: &str, master_secret: &str, user_id: &str) -> Result<Keypair, MpcError> {
    let bytes = decrypt_privkey(hex_ct, master_secret, user_id)?;
    Keypair::try_from(bytes.as_slice()).map_err(|e| {
        tracing::error!("Keypair::try_from failed: {e}");
        MpcError::Internal
    })
}

pub fn verify_escrow_hmac(from_pubkey: &str, hmac_hex: &str, secret: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(b":");
    hasher.update(from_pubkey.as_bytes());
    let expected = hex::encode(hasher.finalize());
    expected.len() == hmac_hex.len()
        && expected
            .bytes()
            .zip(hmac_hex.bytes())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
}

pub async fn create_wallet(user_id: &str, config: &Config) -> Result<String, MpcError> {
    if user_id.is_empty() {
        return Err(MpcError::BadRequest(String::from("user_id must not be empty")));
    }

    let uid = Uuid::parse_str(user_id).map_err(|_| {
        MpcError::BadRequest(format!("user_id is not a valid UUID: {user_id}"))
    })?;

    if let Some(row) = store::wallet_keys::find_wallet_key_by_user_id(uid)
        .await
        .map_err(|e| {
            tracing::error!("DB lookup failed: {e}");
            MpcError::Internal
        })?
    {
        tracing::info!(user_id, pubkey = %row.pubkey, "wallet already exists, returning existing pubkey");
        return Ok(row.pubkey);
    }

    let keypair       = derive_keypair(&config.master_secret, user_id);
    let pubkey_b58    = keypair.pubkey().to_string();
    let enc_privkey   = encrypt_privkey(&keypair.to_bytes(), &config.master_secret, user_id)?;

    store::wallet_keys::insert_wallet_key_with_privkey(uid, &pubkey_b58, &enc_privkey)
        .await
        .map_err(|e| {
            tracing::error!("insert_wallet_key_with_privkey failed: {e}");
            MpcError::Internal
        })?;

    tracing::info!(user_id, pubkey = %pubkey_b58, "wallet created");
    Ok(pubkey_b58)
}

pub async fn execute_transfer(req: TransferRequest, config: &Config) -> Result<String, MpcError> {
    if matches!(req.signer, SignerInfo::Escrow) {
        let proof = req.escrow_hmac.as_deref().ok_or(MpcError::Unauthorized)?;
        if !verify_escrow_hmac(&req.from, proof, &config.escrow_hmac_secret) {
            return Err(MpcError::Unauthorized);
        }
    }

    if req.amount <= 0 {
        return Err(MpcError::BadRequest(String::from("amount must be positive")));
    }
    if req.from.is_empty() || req.to.is_empty() {
        return Err(MpcError::BadRequest(String::from("from and to are required")));
    }

    let from_pubkey = Pubkey::from_str(&req.from)
        .map_err(|_| MpcError::BadRequest(format!("Invalid 'from' address: {}", req.from)))?;
    let to_pubkey = Pubkey::from_str(&req.to)
        .map_err(|_| MpcError::BadRequest(format!("Invalid 'to' address: {}", req.to)))?;
    let payer_pubkey = Pubkey::from_str(&req.payer)
        .map_err(|_| MpcError::BadRequest(format!("Invalid 'payer' address: {}", req.payer)))?;

    let signing_keypair: Keypair = match &req.signer {
        SignerInfo::User { user_id } => {
            let uid = Uuid::parse_str(user_id).map_err(|_| {
                MpcError::BadRequest(format!("user_id is not a valid UUID: {user_id}"))
            })?;
            let row = store::wallet_keys::find_wallet_key_by_user_id(uid)
                .await
                .map_err(|e| {
                    tracing::error!("DB lookup failed: {e}");
                    MpcError::Internal
                })?
                .ok_or_else(|| MpcError::KeyNotFound(user_id.clone()))?;
            let enc = row.encrypted_private_key.ok_or_else(|| {
                MpcError::KeyNotFound(format!("No encrypted_private_key for user {user_id}"))
            })?;
            load_keypair_from_enc(&enc, &config.master_secret, user_id)?
        }
        SignerInfo::Escrow => {
            let bytes = bs58::decode(&config.escrow_private_key)
                .into_vec()
                .map_err(|e| {
                    tracing::error!("base58 decode of escrow keypair failed: {e}");
                    MpcError::Internal
                })?;
            Keypair::try_from(bytes.as_slice()).map_err(|e| {
                tracing::error!("escrow Keypair::try_from failed: {e}");
                MpcError::Internal
            })?
        }
    };

    let instructions = if let Some(mint_str) = &req.mint {
        let mint_pubkey = Pubkey::from_str(mint_str)
            .map_err(|_| MpcError::BadRequest(format!("Invalid mint address: {mint_str}")))?;

        let rpc_pre = RpcClient::new(config.solana_rpc_url.clone());
        let mint_account = rpc_pre.get_account(&mint_pubkey).await.map_err(|e| {
            tracing::error!("get_account (mint) failed: {e}");
            MpcError::BroadcastFailed(e.to_string())
        })?;
        let mint_data = SplMint::unpack(&mint_account.data).map_err(|e| {
            MpcError::BadRequest(format!("Failed to unpack mint account: {e}"))
        })?;
        let decimals = mint_data.decimals;

        let from_ata = get_associated_token_address(&from_pubkey, &mint_pubkey);
        let to_ata   = get_associated_token_address(&to_pubkey,   &mint_pubkey);

        let mut ixs = Vec::new();

        match rpc_pre.get_account(&to_ata).await {
            Ok(_) => {} 
            Err(_) => {
                tracing::info!(%to_ata, "recipient ATA does not exist, adding create instruction");
                ixs.push(create_associated_token_account(
                    &payer_pubkey,  
                    &to_pubkey,     
                    &mint_pubkey,
                    &spl_token::id(),
                ));
            }
        }

        ixs.push(
            spl_token::instruction::transfer_checked(
                &spl_token::id(),
                &from_ata,        
                &mint_pubkey,
                &to_ata,          
                &from_pubkey,     
                &[],
                req.amount as u64,
                decimals,
            )
            .map_err(|e| MpcError::SigningFailed(format!("transfer_checked ix failed: {e}")))?,
        );
        ixs
    } else {
        vec![system_instruction::transfer(&from_pubkey, &to_pubkey, req.amount as u64)]
    };

    let rpc = RpcClient::new(config.solana_rpc_url.clone());

    let recent_blockhash = rpc.get_latest_blockhash().await.map_err(|e| {
        tracing::error!("get_latest_blockhash failed: {e}");
        MpcError::BroadcastFailed(e.to_string())
    })?;

    let message = Message::new_with_blockhash(&instructions, Some(&payer_pubkey), &recent_blockhash);
    let mut tx  = Transaction::new_unsigned(message);
    tx.sign(&[&signing_keypair], recent_blockhash);

    let signature = rpc
        .send_and_confirm_transaction(&tx)
        .await
        .map_err(|e| {
            tracing::error!("send_and_confirm_transaction failed: {e}");
            MpcError::BroadcastFailed(e.to_string())
        })?;

    Ok(signature.to_string())
}

pub async fn sign_and_send(req: SignAndSendRequest, config: &Config) -> Result<String, MpcError> {
    if req.user_id.is_empty() {
        return Err(MpcError::BadRequest(String::from("user_id must not be empty")));
    }
    if req.transaction_base64.is_empty() {
        return Err(MpcError::BadRequest(String::from("transaction_base64 must not be empty")));
    }

    let uid = Uuid::parse_str(&req.user_id).map_err(|_| {
        MpcError::BadRequest(format!("user_id is not a valid UUID: {}", req.user_id))
    })?;
    let row = store::wallet_keys::find_wallet_key_by_user_id(uid)
        .await
        .map_err(|e| {
            tracing::error!("DB lookup failed: {e}");
            MpcError::Internal
        })?
        .ok_or_else(|| MpcError::KeyNotFound(req.user_id.clone()))?;

    let enc = row.encrypted_private_key.ok_or_else(|| {
        MpcError::KeyNotFound(format!("No encrypted_private_key for user {}", req.user_id))
    })?;
    let keypair = load_keypair_from_enc(&enc, &config.master_secret, &req.user_id)?;

    if keypair.pubkey().to_string() != req.wallet_pubkey {
        return Err(MpcError::Unauthorized);
    }

    let tx_bytes = B64.decode(&req.transaction_base64).map_err(|e| {
        MpcError::BadRequest(format!("base64 decode failed: {e}"))
    })?;

    let mut tx: solana_sdk::transaction::VersionedTransaction =
        bincode::deserialize(&tx_bytes).map_err(|e| {
            MpcError::BadRequest(format!("transaction deserialize failed: {e}"))
        })?;

    let message_data = tx.message.serialize();
    let sig = keypair.try_sign_message(&message_data).map_err(|e| {
        MpcError::SigningFailed(e.to_string())
    })?;

    let static_keys = tx.message.static_account_keys();
    let pos = static_keys
        .iter()
        .position(|k| k == &keypair.pubkey())
        .ok_or_else(|| {
            MpcError::SigningFailed(String::from(
                "Signing keypair is not in the transaction's account keys",
            ))
        })?;

    if tx.signatures.len() <= pos {
        tx.signatures
            .resize(pos + 1, solana_sdk::signature::Signature::default());
    }
    tx.signatures[pos] = sig;

    let rpc = RpcClient::new(config.solana_rpc_url.clone());

    let confirmed_sig = rpc
        .send_and_confirm_transaction(&tx)
        .await
        .map_err(|e| {
            tracing::error!("send_and_confirm_transaction failed: {e}");
            MpcError::BroadcastFailed(e.to_string())
        })?;

    Ok(confirmed_sig.to_string())
}
