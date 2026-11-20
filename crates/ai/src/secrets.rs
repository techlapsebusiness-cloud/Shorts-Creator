use std::collections::HashMap;

use crate::KEYCHAIN_SERVICE;
use crate::error::AiError;

pub trait SecretStore {
    fn set(&mut self, account: &str, secret: &str) -> Result<(), AiError>;
    fn get(&self, account: &str) -> Result<Option<String>, AiError>;
}

#[derive(Debug, Default)]
pub struct MemoryStore {
    secrets: HashMap<String, String>,
}

impl SecretStore for MemoryStore {
    fn set(&mut self, account: &str, secret: &str) -> Result<(), AiError> {
        self.secrets.insert(account.to_string(), secret.to_string());
        Ok(())
    }

    fn get(&self, account: &str) -> Result<Option<String>, AiError> {
        Ok(self.secrets.get(account).cloned())
    }
}

fn is_missing(error: &keyring::Error) -> bool {
    matches!(error, keyring::Error::NoEntry)
}

/// OS keychain. Unit tests use [`MemoryStore`] and do not construct this.
#[derive(Debug)]
pub struct OsStore;

impl SecretStore for OsStore {
    fn set(&mut self, account: &str, secret: &str) -> Result<(), AiError> {
        let entry = keyring::Entry::new(KEYCHAIN_SERVICE, account)
            .map_err(|error| AiError::Keychain(error.to_string()))?;
        entry
            .set_password(secret)
            .map_err(|error| AiError::Keychain(error.to_string()))
    }

    fn get(&self, account: &str) -> Result<Option<String>, AiError> {
        let entry = keyring::Entry::new(KEYCHAIN_SERVICE, account)
            .map_err(|error| AiError::Keychain(error.to_string()))?;
        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(error) if is_missing(&error) => Ok(None),
            Err(error) => Err(AiError::Keychain(error.to_string())),
        }
    }
}
