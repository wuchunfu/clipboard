use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use std::fs;
use std::path::Path;

pub struct Crypto {
    cipher: Aes256Gcm,
}

impl Crypto {
    pub fn new<P: AsRef<Path>>(key_path: P) -> Self {
        let key = if key_path.as_ref().exists() {
            let bytes = fs::read(&key_path).expect("Failed to read key file");
            Key::<Aes256Gcm>::from_slice(&bytes).clone()
        } else {
            let mut key = Key::<Aes256Gcm>::default();
            OsRng.fill_bytes(&mut key);
            fs::write(&key_path, key).expect("Failed to write key file");
            key
        };

        Self {
            cipher: Aes256Gcm::new(&key),
        }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String, String> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| e.to_string())?;

        // Format: nonce + ciphertext (base64 encoded)
        let mut combined = nonce.to_vec();
        combined.extend(ciphertext);

        use base64::{engine::general_purpose, Engine as _};
        Ok(general_purpose::STANDARD.encode(combined))
    }

    pub fn decrypt(&self, encrypted_base64: &str) -> Result<String, String> {
        use base64::{engine::general_purpose, Engine as _};
        let decoded = general_purpose::STANDARD
            .decode(encrypted_base64)
            .map_err(|e| e.to_string())?;

        if decoded.len() < 12 {
            return Err("Invalid encrypted data length".to_string());
        }

        let (nonce_bytes, ciphertext) = decoded.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext_bytes = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| e.to_string())?;

        String::from_utf8(plaintext_bytes).map_err(|e| e.to_string())
    }
}
