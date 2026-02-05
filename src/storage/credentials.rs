use crate::error::{CaliError, Result};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use std::fs;
use std::path::PathBuf;

const SERVICE_NAME: &str = "com.github.cali";
const ENCRYPTED_FILE_NAME: &str = "credentials.enc";
const PBKDF2_ITERATIONS: u32 = 100_000;
const APP_SALT: &[u8] = b"cali-secure-storage-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialBackend {
    Keychain,
    EncryptedFile,
}

impl std::fmt::Display for CredentialBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CredentialBackend::Keychain => write!(f, "system keychain"),
            CredentialBackend::EncryptedFile => write!(f, "encrypted file"),
        }
    }
}

pub struct SecureStorage {
    backend: CredentialBackend,
    config_dir: PathBuf,
}

impl SecureStorage {
    pub fn new(config_dir: PathBuf) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let test_account = format!(
            "test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        let backend = if keyring::Entry::new(SERVICE_NAME, &test_account)
            .and_then(|e| e.set_password("test"))
            .and_then(|_| keyring::Entry::new(SERVICE_NAME, &test_account))
            .and_then(|e| e.delete_credential())
            .is_ok()
        {
            CredentialBackend::Keychain
        } else {
            CredentialBackend::EncryptedFile
        };

        Self {
            backend,
            config_dir,
        }
    }

    #[cfg(test)]
    pub fn new_for_testing(config_dir: PathBuf) -> Self {
        Self {
            backend: CredentialBackend::EncryptedFile,
            config_dir,
        }
    }

    pub fn backend(&self) -> CredentialBackend {
        self.backend
    }

    pub fn store_url(&self, calendar_name: &str, url: &str) -> Result<()> {
        match self.backend {
            CredentialBackend::Keychain => self.store_in_keychain(calendar_name, url),
            CredentialBackend::EncryptedFile => self.store_in_file(calendar_name, url),
        }
    }

    pub fn get_url(&self, calendar_name: &str) -> Result<Option<String>> {
        match self.backend {
            CredentialBackend::Keychain => self.get_from_keychain(calendar_name),
            CredentialBackend::EncryptedFile => self.get_from_file(calendar_name),
        }
    }

    pub fn delete_url(&self, calendar_name: &str) -> Result<()> {
        match self.backend {
            CredentialBackend::Keychain => self.delete_from_keychain(calendar_name),
            CredentialBackend::EncryptedFile => self.delete_from_file(calendar_name),
        }
    }

    fn keychain_account(calendar_name: &str) -> String {
        format!("calendar:{}", calendar_name)
    }

    fn store_in_keychain(&self, calendar_name: &str, url: &str) -> Result<()> {
        let entry = keyring::Entry::new(SERVICE_NAME, &Self::keychain_account(calendar_name))
            .map_err(|e| {
                CaliError::credential_storage("Failed to create keychain entry".to_string(), e)
            })?;

        entry.set_password(url).map_err(|e| {
            CaliError::credential_storage("Failed to store credential in keychain".to_string(), e)
        })?;

        Ok(())
    }

    fn get_from_keychain(&self, calendar_name: &str) -> Result<Option<String>> {
        let entry = keyring::Entry::new(SERVICE_NAME, &Self::keychain_account(calendar_name))
            .map_err(|e| {
                CaliError::credential_storage("Failed to create keychain entry".to_string(), e)
            })?;

        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(CaliError::credential_storage(
                "Failed to retrieve credential from keychain".to_string(),
                e,
            )),
        }
    }

    fn delete_from_keychain(&self, calendar_name: &str) -> Result<()> {
        let entry = keyring::Entry::new(SERVICE_NAME, &Self::keychain_account(calendar_name))
            .map_err(|e| {
                CaliError::credential_storage("Failed to create keychain entry".to_string(), e)
            })?;

        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(CaliError::credential_storage(
                "Failed to delete credential from keychain".to_string(),
                e,
            )),
        }
    }

    fn credentials_file_path(&self) -> PathBuf {
        self.config_dir.join(ENCRYPTED_FILE_NAME)
    }

    fn derive_key(&self) -> Result<[u8; 32]> {
        let machine_id = machine_uid::get().map_err(|e| {
            CaliError::credential_storage(
                format!("Failed to get machine ID: {}", e),
                std::io::Error::new(std::io::ErrorKind::Other, "machine ID error"),
            )
        })?;

        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(
            machine_id.as_bytes(),
            APP_SALT,
            PBKDF2_ITERATIONS,
            &mut key,
        );

        Ok(key)
    }

    fn store_in_file(&self, calendar_name: &str, url: &str) -> Result<()> {
        let mut credentials = self.load_credentials()?;
        credentials.insert(calendar_name.to_string(), url.to_string());
        self.save_credentials(&credentials)
    }

    fn get_from_file(&self, calendar_name: &str) -> Result<Option<String>> {
        let credentials = self.load_credentials()?;
        Ok(credentials.get(calendar_name).cloned())
    }

    fn delete_from_file(&self, calendar_name: &str) -> Result<()> {
        let mut credentials = self.load_credentials()?;
        credentials.remove(calendar_name);
        self.save_credentials(&credentials)
    }

    fn load_credentials(&self) -> Result<std::collections::HashMap<String, String>> {
        let file_path = self.credentials_file_path();

        if !file_path.exists() {
            return Ok(std::collections::HashMap::new());
        }

        let encrypted_data = fs::read(&file_path).map_err(|e| {
            CaliError::credential_storage(
                format!("Failed to read credentials file: {}", file_path.display()),
                e,
            )
        })?;

        if encrypted_data.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        if encrypted_data.len() < 29 {
            return Err(CaliError::credential_storage(
                "Credentials file is corrupted (too small)".to_string(),
                std::io::Error::new(std::io::ErrorKind::InvalidData, "file too small"),
            ));
        }

        let version = encrypted_data[0];
        if version != 1 {
            return Err(CaliError::credential_storage(
                format!("Unsupported credentials file version: {}", version),
                std::io::Error::new(std::io::ErrorKind::InvalidData, "unsupported version"),
            ));
        }

        let nonce_bytes = &encrypted_data[1..13];
        let ciphertext = &encrypted_data[13..];

        let key = self.derive_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| {
            CaliError::credential_storage("Failed to create cipher".to_string(), e)
        })?;

        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
            CaliError::credential_storage(
                format!(
                    "Failed to decrypt credentials (incorrect key or corrupted data): {}",
                    e
                ),
                std::io::Error::new(std::io::ErrorKind::InvalidData, "decryption failed"),
            )
        })?;

        let credentials: std::collections::HashMap<String, String> =
            serde_json::from_slice(&plaintext).map_err(|e| {
                CaliError::credential_storage("Failed to parse decrypted credentials".to_string(), e)
            })?;

        Ok(credentials)
    }

    fn save_credentials(
        &self,
        credentials: &std::collections::HashMap<String, String>,
    ) -> Result<()> {
        let plaintext = serde_json::to_vec(credentials).map_err(|e| {
            CaliError::credential_storage("Failed to serialize credentials".to_string(), e)
        })?;

        let key = self.derive_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| {
            CaliError::credential_storage("Failed to create cipher".to_string(), e)
        })?;

        let nonce_bytes = self.generate_nonce();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).map_err(|e| {
            CaliError::credential_storage(
                format!("Failed to encrypt credentials: {}", e),
                std::io::Error::new(std::io::ErrorKind::Other, "encryption failed"),
            )
        })?;

        let mut encrypted_data = Vec::with_capacity(1 + 12 + ciphertext.len());
        encrypted_data.push(1);
        encrypted_data.extend_from_slice(&nonce_bytes);
        encrypted_data.extend_from_slice(&ciphertext);

        fs::create_dir_all(&self.config_dir).map_err(|e| {
            CaliError::credential_storage(
                format!(
                    "Failed to create config directory: {}",
                    self.config_dir.display()
                ),
                e,
            )
        })?;

        let file_path = self.credentials_file_path();
        fs::write(&file_path, encrypted_data).map_err(|e| {
            CaliError::credential_storage(
                format!("Failed to write credentials file: {}", file_path.display()),
                e,
            )
        })?;

        Ok(())
    }

    fn generate_nonce(&self) -> [u8; 12] {
        use rand::RngCore;

        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        nonce
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_backend_display() {
        assert_eq!(CredentialBackend::Keychain.to_string(), "system keychain");
        assert_eq!(
            CredentialBackend::EncryptedFile.to_string(),
            "encrypted file"
        );
    }

    #[test]
    fn test_encrypted_file_store_and_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SecureStorage::new_for_testing(temp_dir.path().to_path_buf());

        storage
            .store_url("test-calendar", "https://example.com/calendar.ics")
            .unwrap();

        let retrieved = storage.get_url("test-calendar").unwrap();
        assert_eq!(retrieved, Some("https://example.com/calendar.ics".to_string()));
    }

    #[test]
    fn test_encrypted_file_delete() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SecureStorage::new_for_testing(temp_dir.path().to_path_buf());

        storage
            .store_url("test-calendar", "https://example.com/calendar.ics")
            .unwrap();

        storage.delete_url("test-calendar").unwrap();

        let retrieved = storage.get_url("test-calendar").unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_encrypted_file_multiple_calendars() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SecureStorage::new_for_testing(temp_dir.path().to_path_buf());

        storage
            .store_url("calendar1", "https://example.com/cal1.ics")
            .unwrap();
        storage
            .store_url("calendar2", "https://example.com/cal2.ics")
            .unwrap();

        let cal1 = storage.get_url("calendar1").unwrap();
        let cal2 = storage.get_url("calendar2").unwrap();

        assert_eq!(cal1, Some("https://example.com/cal1.ics".to_string()));
        assert_eq!(cal2, Some("https://example.com/cal2.ics".to_string()));
    }

    #[test]
    fn test_encrypted_file_empty_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SecureStorage::new_for_testing(temp_dir.path().to_path_buf());

        let retrieved = storage.get_url("nonexistent").unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_encrypted_file_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SecureStorage::new_for_testing(temp_dir.path().to_path_buf());

        storage
            .store_url("test", "https://example.com/old.ics")
            .unwrap();
        storage
            .store_url("test", "https://example.com/new.ics")
            .unwrap();

        let retrieved = storage.get_url("test").unwrap();
        assert_eq!(retrieved, Some("https://example.com/new.ics".to_string()));
    }

    #[test]
    fn test_keychain_account_format() {
        assert_eq!(
            SecureStorage::keychain_account("work"),
            "calendar:work".to_string()
        );
        assert_eq!(
            SecureStorage::keychain_account("personal"),
            "calendar:personal".to_string()
        );
    }

    #[test]
    fn test_nonce_generation_randomness() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SecureStorage::new_for_testing(temp_dir.path().to_path_buf());

        let nonce1 = storage.generate_nonce();
        let nonce2 = storage.generate_nonce();

        assert_ne!(nonce1, nonce2);
        assert_eq!(nonce1.len(), 12);
        assert_eq!(nonce2.len(), 12);
    }
}
