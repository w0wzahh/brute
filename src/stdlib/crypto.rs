use std::collections::HashMap;
use sha2::{Sha256, Sha512, Digest};
use hmac::{Hmac, Mac};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::RngCore;
use crate::interpreter::Value;
use crate::error::{BruteError, Result};

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> Result<Vec<u8>> {
    if s.len() % 2 != 0 {
        return Err(BruteError::ValueError("invalid hex length".into()));
    }
    (0..s.len()).step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16)
            .map_err(|e| BruteError::ValueError(format!("invalid hex: {}", e))))
        .collect()
}

fn text(args: &[Value], i: usize) -> String {
    args.get(i).map(|v| v.display()).unwrap_or_default()
}

fn aes_cipher(args: &[Value], key_i: usize, nonce_i: usize) -> Result<(Aes256Gcm, Vec<u8>)> {
    let key_bytes = hex_decode(&text(args, key_i))?;
    let nonce_bytes = hex_decode(&text(args, nonce_i))?;
    if nonce_bytes.len() != 12 {
        return Err(BruteError::ValueError("AES-256-GCM nonce must be 12 bytes (24 hex chars)".into()));
    }
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key_bytes));
    Ok((cipher, nonce_bytes))
}

/// Returns a map of name → Value for the crypto module.
pub fn module() -> HashMap<String, Value> {
    let mut m: HashMap<String, Value> = HashMap::new();

    macro_rules! f {
        ($name:expr, $body:expr) => {
            m.insert($name.into(), Value::NativeFunction {
                name: concat!("crypto::", $name).into(),
                func: $body,
            });
        };
    }

    // ── hashing ──────────────────────────────────────────────────────────
    f!("sha256", |_, a| {
        let mut h = Sha256::new();
        h.update(text(&a, 0).as_bytes());
        Ok(Value::String(hex_encode(&h.finalize())))
    });
    f!("sha512", |_, a| {
        let mut h = Sha512::new();
        h.update(text(&a, 0).as_bytes());
        Ok(Value::String(hex_encode(&h.finalize())))
    });
    f!("hmac_sha256", |_, a| {
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(text(&a, 0).as_bytes())
            .map_err(|e| BruteError::ValueError(format!("invalid HMAC key: {}", e)))?;
        mac.update(text(&a, 1).as_bytes());
        Ok(Value::String(hex_encode(&mac.finalize().into_bytes())))
    });

    // ── AES-256-GCM ─────────────────────────────────────────────────────
    f!("generate_aes_key", |_, _| {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Ok(Value::String(hex_encode(&key)))
    });
    f!("generate_nonce", |_, _| {
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        Ok(Value::String(hex_encode(&nonce)))
    });
    f!("encrypt", |_, a| {
        let (cipher, nonce) = aes_cipher(&a, 0, 1)?;
        let ct = cipher.encrypt(Nonce::from_slice(&nonce), text(&a, 2).as_bytes())
            .map_err(|e| BruteError::RuntimeError(format!("encryption failed: {}", e)))?;
        Ok(Value::String(hex_encode(&ct)))
    });
    f!("decrypt", |_, a| {
        let (cipher, nonce) = aes_cipher(&a, 0, 1)?;
        let ct = hex_decode(&text(&a, 2))?;
        let pt = cipher.decrypt(Nonce::from_slice(&nonce), ct.as_ref())
            .map_err(|_| BruteError::ValueError("decryption failed (bad key, nonce, or ciphertext)".into()))?;
        String::from_utf8(pt).map(Value::String)
            .map_err(|e| BruteError::ValueError(format!("decrypted bytes are not UTF-8: {}", e)))
    });

    // ── encoding ─────────────────────────────────────────────────────────
    f!("hex_encode", |_, a| Ok(Value::String(hex_encode(text(&a, 0).as_bytes()))));
    f!("hex_decode", |_, a| {
        String::from_utf8(hex_decode(&text(&a, 0))?)
            .map(Value::String)
            .map_err(|e| BruteError::ValueError(format!("invalid UTF-8: {}", e)))
    });
    f!("base64_encode", |_, a| Ok(Value::String(B64.encode(text(&a, 0).as_bytes()))));
    f!("base64_decode", |_, a| {
        let bytes = B64.decode(text(&a, 0))
            .map_err(|e| BruteError::ValueError(format!("invalid base64: {}", e)))?;
        String::from_utf8(bytes).map(Value::String)
            .map_err(|e| BruteError::ValueError(format!("invalid UTF-8: {}", e)))
    });
    f!("random_bytes", |_, a| {
        let n = match a.get(0) { Some(Value::Int(n)) => (*n).max(0) as usize, _ => 16 };
        let mut buf = vec![0u8; n];
        rand::thread_rng().fill_bytes(&mut buf);
        Ok(Value::String(hex_encode(&buf)))
    });

    m
}
