use crate::error::{SecurityError, SecurityResult};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::{rand_core::OsRng as ArgonOsRng, SaltString}};
use ring::{digest, hmac, pbkdf2, rand::SecureRandom};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

#[derive(Clone)]
pub struct EncryptionService {
    master_key: Arc<[u8; 32]>,
    key_cache: Arc<RwLock<HashMap<String, Arc<[u8; 32]>>>>,
    algorithm: String,
    key_rotation_days: u32,
}

impl EncryptionService {
    pub fn new(master_key: [u8; 32], algorithm: String, key_rotation_days: u32) -> Self {
        Self {
            master_key: Arc::new(master_key),
            key_cache: Arc::new(RwLock::new(HashMap::new())),
            algorithm,
            key_rotation_days,
        }
    }

    /// Initialize from configuration
    pub async fn from_config(
        master_key_id: &str,
        algorithm: &str,
        key_rotation_days: u32,
    ) -> SecurityResult<Self> {
        // In production, this would retrieve the master key from a KMS
        // For now, we'll derive it from the key ID
        let master_key = Self::derive_master_key(master_key_id)?;
        
        Ok(Self::new(master_key, algorithm.to_string(), key_rotation_days))
    }

    /// Encrypt data using AES-256-GCM
    pub async fn encrypt_data(&self, data: &[u8]) -> SecurityResult<Vec<u8>> {
        match self.algorithm.as_str() {
            "AES-256-GCM" => self.encrypt_aes_gcm(data).await,
            _ => Err(SecurityError::Encryption(format!("Unsupported algorithm: {}", self.algorithm))),
        }
    }

    /// Decrypt data using AES-256-GCM
    pub async fn decrypt_data(&self, encrypted_data: &[u8]) -> SecurityResult<Vec<u8>> {
        match self.algorithm.as_str() {
            "AES-256-GCM" => self.decrypt_aes_gcm(encrypted_data).await,
            _ => Err(SecurityError::Encryption(format!("Unsupported algorithm: {}", self.algorithm))),
        }
    }

    /// Encrypt data with a tenant-specific key
    pub async fn encrypt_tenant_data(&self, tenant_id: &str, data: &[u8]) -> SecurityResult<Vec<u8>> {
        let tenant_key = self.get_tenant_key(tenant_id).await?;
        self.encrypt_with_key(&tenant_key, data).await
    }

    /// Decrypt data with a tenant-specific key
    pub async fn decrypt_tenant_data(&self, tenant_id: &str, encrypted_data: &[u8]) -> SecurityResult<Vec<u8>> {
        let tenant_key = self.get_tenant_key(tenant_id).await?;
        self.decrypt_with_key(&tenant_key, encrypted_data).await
    }

    /// Hash a password using Argon2
    pub async fn hash_password(&self, password: &str) -> SecurityResult<String> {
        let salt = SaltString::generate(&mut ArgonOsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| SecurityError::Encryption(format!("Password hashing failed: {}", e)))?;
        
        Ok(password_hash.to_string())
    }

    /// Verify a password against its hash
    pub async fn verify_password(&self, password: &str, hash: &str) -> SecurityResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| SecurityError::Encryption(format!("Invalid password hash: {}", e)))?;
        
        let argon2 = Argon2::default();
        
        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Generate a secure random key
    pub async fn generate_key(&self) -> SecurityResult<[u8; 32]> {
        let mut key = [0u8; 32];
        let rng = ring::rand::SystemRandom::new();
        rng.fill(&mut key)
            .map_err(|e| SecurityError::Encryption(format!("Key generation failed: {:?}", e)))?;
        Ok(key)
    }

    /// Generate a secure random token
    pub async fn generate_token(&self, length: usize) -> SecurityResult<String> {
        let mut token_bytes = vec![0u8; length];
        let rng = ring::rand::SystemRandom::new();
        rng.fill(&mut token_bytes)
            .map_err(|e| SecurityError::Encryption(format!("Token generation failed: {:?}", e)))?;
        
        Ok(hex::encode(token_bytes))
    }

    /// Create HMAC signature
    pub async fn create_hmac(&self, data: &[u8], key: &[u8]) -> SecurityResult<Vec<u8>> {
        let key = hmac::Key::new(hmac::HMAC_SHA256, key);
        let signature = hmac::sign(&key, data);
        Ok(signature.as_ref().to_vec())
    }

    /// Verify HMAC signature
    pub async fn verify_hmac(&self, data: &[u8], signature: &[u8], key: &[u8]) -> SecurityResult<bool> {
        let key = hmac::Key::new(hmac::HMAC_SHA256, key);
        match hmac::verify(&key, data, signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Derive key using PBKDF2
    pub async fn derive_key(&self, password: &str, salt: &[u8], iterations: u32) -> SecurityResult<[u8; 32]> {
        let mut key = [0u8; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            std::num::NonZeroU32::new(iterations).unwrap(),
            salt,
            password.as_bytes(),
            &mut key,
        );
        Ok(key)
    }

    /// Encrypt file data
    pub async fn encrypt_file(&self, file_data: &[u8], tenant_id: &str) -> SecurityResult<EncryptedFile> {
        let tenant_key = self.get_tenant_key(tenant_id).await?;
        let encrypted_data = self.encrypt_with_key(&tenant_key, file_data).await?;
        
        // Generate file-specific metadata
        let file_id = Uuid::new_v4().to_string();
        let checksum = self.calculate_checksum(file_data);
        
        Ok(EncryptedFile {
            file_id,
            encrypted_data,
            checksum,
            algorithm: self.algorithm.clone(),
            tenant_id: tenant_id.to_string(),
            created_at: chrono::Utc::now(),
        })
    }

    /// Decrypt file data
    pub async fn decrypt_file(&self, encrypted_file: &EncryptedFile) -> SecurityResult<Vec<u8>> {
        let tenant_key = self.get_tenant_key(&encrypted_file.tenant_id).await?;
        let decrypted_data = self.decrypt_with_key(&tenant_key, &encrypted_file.encrypted_data).await?;
        
        // Verify checksum
        let calculated_checksum = self.calculate_checksum(&decrypted_data);
        if calculated_checksum != encrypted_file.checksum {
            return Err(SecurityError::Encryption("File integrity check failed".to_string()));
        }
        
        Ok(decrypted_data)
    }

    /// Rotate encryption keys
    pub async fn rotate_keys(&self, tenant_id: &str) -> SecurityResult<()> {
        info!(tenant_id = %tenant_id, "Starting key rotation");
        
        // Generate new key
        let new_key = self.generate_key().await?;
        
        // Update key cache
        let mut cache = self.key_cache.write().await;
        cache.insert(tenant_id.to_string(), Arc::new(new_key));
        
        info!(tenant_id = %tenant_id, "Key rotation completed");
        Ok(())
    }

    /// Get encryption status for a tenant
    pub async fn get_encryption_status(&self, tenant_id: &str) -> SecurityResult<EncryptionStatus> {
        let cache = self.key_cache.read().await;
        let has_key = cache.contains_key(tenant_id);
        
        Ok(EncryptionStatus {
            tenant_id: tenant_id.to_string(),
            algorithm: self.algorithm.clone(),
            key_exists: has_key,
            key_rotation_days: self.key_rotation_days,
            last_rotation: None, // This would be tracked in a database
        })
    }

    // Private helper methods

    async fn encrypt_aes_gcm(&self, data: &[u8]) -> SecurityResult<Vec<u8>> {
        let key = Key::<Aes256Gcm>::from_slice(&*self.master_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| SecurityError::Encryption(format!("AES-GCM encryption failed: {}", e)))?;
        
        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    async fn decrypt_aes_gcm(&self, encrypted_data: &[u8]) -> SecurityResult<Vec<u8>> {
        if encrypted_data.len() < 12 {
            return Err(SecurityError::Encryption("Invalid encrypted data length".to_string()));
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let key = Key::<Aes256Gcm>::from_slice(&*self.master_key);
        let cipher = Aes256Gcm::new(key);
        
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| SecurityError::Encryption(format!("AES-GCM decryption failed: {}", e)))?;
        
        Ok(plaintext)
    }

    async fn encrypt_with_key(&self, key: &[u8; 32], data: &[u8]) -> SecurityResult<Vec<u8>> {
        let aes_key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(aes_key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| SecurityError::Encryption(format!("Encryption failed: {}", e)))?;
        
        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    async fn decrypt_with_key(&self, key: &[u8; 32], encrypted_data: &[u8]) -> SecurityResult<Vec<u8>> {
        if encrypted_data.len() < 12 {
            return Err(SecurityError::Encryption("Invalid encrypted data length".to_string()));
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let aes_key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(aes_key);
        
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| SecurityError::Encryption(format!("Decryption failed: {}", e)))?;
        
        Ok(plaintext)
    }

    async fn get_tenant_key(&self, tenant_id: &str) -> SecurityResult<Arc<[u8; 32]>> {
        // Check cache first
        {
            let cache = self.key_cache.read().await;
            if let Some(key) = cache.get(tenant_id) {
                return Ok(key.clone());
            }
        }
        
        // Generate tenant-specific key if not in cache
        let tenant_key = self.derive_tenant_key(tenant_id)?;
        
        // Cache the key
        {
            let mut cache = self.key_cache.write().await;
            let key_arc = Arc::new(tenant_key);
            cache.insert(tenant_id.to_string(), key_arc.clone());
            Ok(key_arc)
        }
    }

    fn derive_tenant_key(&self, tenant_id: &str) -> SecurityResult<[u8; 32]> {
        let mut tenant_key = [0u8; 32];
        let salt = format!("adx-core-tenant-{}", tenant_id);
        
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            std::num::NonZeroU32::new(100_000).unwrap(),
            salt.as_bytes(),
            &*self.master_key,
            &mut tenant_key,
        );
        
        Ok(tenant_key)
    }

    fn derive_master_key(key_id: &str) -> SecurityResult<[u8; 32]> {
        // In production, this would retrieve from KMS
        // For development, derive from key ID
        let mut master_key = [0u8; 32];
        let salt = b"adx-core-master-salt";
        
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            std::num::NonZeroU32::new(100_000).unwrap(),
            salt,
            key_id.as_bytes(),
            &mut master_key,
        );
        
        Ok(master_key)
    }

    fn calculate_checksum(&self, data: &[u8]) -> String {
        let digest = digest::digest(&digest::SHA256, data);
        hex::encode(digest.as_ref())
    }
}

// Supporting types

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptedFile {
    pub file_id: String,
    pub encrypted_data: Vec<u8>,
    pub checksum: String,
    pub algorithm: String,
    pub tenant_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptionStatus {
    pub tenant_id: String,
    pub algorithm: String,
    pub key_exists: bool,
    pub key_rotation_days: u32,
    pub last_rotation: Option<chrono::DateTime<chrono::Utc>>,
}

// External crate for hex encoding
use hex;