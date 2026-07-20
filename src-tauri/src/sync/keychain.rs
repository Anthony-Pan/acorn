use keyring::Entry;

use super::error::{SyncError, SyncResult};

/// Same keychain service as the AI provider keys — one Acorn namespace,
/// bundle id `app.acorn.desktop`. Only the OAuth *refresh* token is stored;
/// short-lived access tokens stay in memory and are re-minted on demand.
const SERVICE: &str = "app.acorn.desktop";

fn account_key(provider: &str, account_id: &str) -> String {
    format!("oauth:{provider}:{account_id}")
}

pub fn save_refresh_token(provider: &str, account_id: &str, token: &str) -> SyncResult<()> {
    let entry = Entry::new(SERVICE, &account_key(provider, account_id))?;
    entry.set_password(token)?;
    Ok(())
}

pub fn load_refresh_token(provider: &str, account_id: &str) -> SyncResult<Option<String>> {
    let entry = Entry::new(SERVICE, &account_key(provider, account_id))?;
    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(SyncError::Keychain(err.to_string())),
    }
}

pub fn delete_refresh_token(provider: &str, account_id: &str) -> SyncResult<()> {
    let entry = Entry::new(SERVICE, &account_key(provider, account_id))?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(SyncError::Keychain(err.to_string())),
    }
}
