//! Subscription links are credentials: a Canvas, Google, or Outlook feed URL
//! carries a private token. They live only in the OS credential store.
//!
//! Nothing here logs, formats, or returns a secret inside an error message.
use std::sync::Mutex;

const SERVICE: &str = "Temporal Engine calendar subscription";

pub trait Vault: Send + Sync {
    fn store(&self, key: &str, secret: &str) -> Result<(), String>;
    /// `Ok(None)` when no credential exists for the key.
    fn load(&self, key: &str) -> Result<Option<String>, String>;
    /// Removing an absent credential is not an error.
    fn remove(&self, key: &str) -> Result<(), String>;
}

/// In-process vault for tests and the non-Windows development build.
#[derive(Default)]
pub struct MemoryVault(Mutex<std::collections::BTreeMap<String, String>>);

impl Vault for MemoryVault {
    fn store(&self, key: &str, secret: &str) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|_| "Credential store unavailable.".to_string())?
            .insert(key.into(), secret.into());
        Ok(())
    }
    fn load(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self
            .0
            .lock()
            .map_err(|_| "Credential store unavailable.".to_string())?
            .get(key)
            .cloned())
    }
    fn remove(&self, key: &str) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|_| "Credential store unavailable.".to_string())?
            .remove(key);
        Ok(())
    }
}

/// Windows Credential Manager, one generic credential per subscription.
#[cfg(windows)]
pub struct WindowsVault;

#[cfg(windows)]
impl Vault for WindowsVault {
    fn store(&self, key: &str, secret: &str) -> Result<(), String> {
        keyring::Entry::new(SERVICE, key)
            .and_then(|entry| entry.set_password(secret))
            .map_err(|_| "Windows Credential Manager refused to store the link.".to_string())
    }
    fn load(&self, key: &str) -> Result<Option<String>, String> {
        let entry = keyring::Entry::new(SERVICE, key)
            .map_err(|_| "Windows Credential Manager is unavailable.".to_string())?;
        match entry.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err("Windows Credential Manager could not read the link.".into()),
        }
    }
    fn remove(&self, key: &str) -> Result<(), String> {
        let entry = keyring::Entry::new(SERVICE, key)
            .map_err(|_| "Windows Credential Manager is unavailable.".to_string())?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err("Windows Credential Manager could not remove the link.".into()),
        }
    }
}

/// The vault this build uses.
pub fn system() -> Box<dyn Vault> {
    #[cfg(windows)]
    {
        Box::new(WindowsVault)
    }
    #[cfg(not(windows))]
    {
        let _ = SERVICE;
        Box::new(MemoryVault::default())
    }
}

/// Accept `https://` or `webcal://` links only; return the fetchable URL and
/// a display host. The error never echoes the link.
pub fn normalize_feed_url(value: &str) -> Result<(String, String), String> {
    let value = value.trim();
    let rest = if let Some(rest) = value.strip_prefix("webcal://") {
        rest
    } else if let Some(rest) = value.strip_prefix("https://") {
        rest
    } else {
        return Err("Paste an https:// or webcal:// calendar link.".into());
    };
    if value.len() > 4096 || value.chars().any(char::is_whitespace) {
        return Err("That calendar link is not valid.".into());
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let host = authority
        .rsplit('@')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if host.is_empty()
        || !host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".-:".contains(c))
    {
        return Err("That calendar link has no valid host.".into());
    }
    Ok((format!("https://{rest}"), host))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_vault_stores_reads_and_removes() {
        let vault = MemoryVault::default();
        assert_eq!(vault.load("a").unwrap(), None);
        vault.store("a", "synthetic-secret").unwrap();
        assert_eq!(
            vault.load("a").unwrap().as_deref(),
            Some("synthetic-secret")
        );
        vault.remove("a").unwrap();
        vault.remove("a").unwrap();
        assert_eq!(vault.load("a").unwrap(), None);
    }

    #[test]
    fn feed_links_are_normalized_without_echoing_secrets() {
        let (url, host) =
            normalize_feed_url("webcal://calendar.example.invalid/feed/token123.ics").unwrap();
        assert_eq!(url, "https://calendar.example.invalid/feed/token123.ics");
        assert_eq!(host, "calendar.example.invalid");
        let (_, host) =
            normalize_feed_url("https://user:pw@Example.Invalid:8443/x?k=token123").unwrap();
        assert_eq!(host, "example.invalid:8443");
        let error = normalize_feed_url("http://example.invalid/token123").unwrap_err();
        assert!(!error.contains("token123"));
        assert!(normalize_feed_url("https:///nohost").is_err());
        assert!(normalize_feed_url("https://bad host/x").is_err());
    }
}
