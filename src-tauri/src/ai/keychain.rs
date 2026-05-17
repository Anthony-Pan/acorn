use keyring::Entry;

use super::error::{ProviderError, ProviderResult};

const SERVICE: &str = "app.acorn.desktop";

pub fn save_api_key(provider_id: &str, api_key: &str) -> ProviderResult<()> {
    let entry = Entry::new(SERVICE, provider_id)?;
    entry.set_password(api_key)?;
    Ok(())
}

pub fn load_api_key(provider_id: &str) -> ProviderResult<Option<String>> {
    let entry = Entry::new(SERVICE, provider_id)?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(ProviderError::Keychain(err.to_string())),
    }
}

pub fn delete_api_key(provider_id: &str) -> ProviderResult<()> {
    let entry = Entry::new(SERVICE, provider_id)?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(ProviderError::Keychain(err.to_string())),
    }
}
