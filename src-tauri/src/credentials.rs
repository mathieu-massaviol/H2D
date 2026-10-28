use std::sync::Arc;

use keyring_core::{CredentialStore as KeyringStore, Error as KeyringError};
use tauri::State;

const SERVICE: &str = "hal2datacite";

pub struct CredentialStore(pub Arc<KeyringStore>);

pub(crate) fn store_key(api: &str, env: &str, field: &str) -> String {
    format!("{api}_{env}_{field}")
}

pub(crate) fn validate_env(env: &str) -> Result<(), String> {
    match env {
        "prod" | "test" => Ok(()),
        other => Err(format!("unknown env: {other}")),
    }
}

/// Non-sensitive credential info returned to the frontend.
#[derive(serde::Serialize, Debug, PartialEq)]
pub struct HalCredentialInfo {
    pub(crate) login: String,
}

/// Non-sensitive credential info returned to the frontend.
#[derive(serde::Serialize, Debug, PartialEq)]
pub struct DataciteCredentialInfo {
    pub(crate) repository_id: String,
}

/// Reads a single secret from the OS credential store. Returns `Err` if the key is absent.
pub(crate) fn get_vault_secret(
    vault: &KeyringStore,
    api: &str,
    env: &str,
    field: &str,
) -> Result<String, String> {
    let entry = vault
        .build(SERVICE, &store_key(api, env, field), None)
        .map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(value) => Ok(value),
        Err(KeyringError::NoEntry) => Err(format!("{api} {field} not configured")),
        Err(e) => Err(e.to_string()),
    }
}

/// Reads an optional secret from the OS credential store. Returns `Ok(None)` if the key is absent.
fn get_optional_vault_secret(
    vault: &KeyringStore,
    api: &str,
    env: &str,
    field: &str,
) -> Result<Option<String>, String> {
    let entry = vault
        .build(SERVICE, &store_key(api, env, field), None)
        .map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

fn set_vault_secret(
    vault: &KeyringStore,
    api: &str,
    env: &str,
    field: &str,
    value: &str,
) -> Result<(), String> {
    vault
        .build(SERVICE, &store_key(api, env, field), None)
        .map_err(|e| e.to_string())?
        .set_password(value)
        .map_err(|e| e.to_string())
}

pub(crate) fn save_hal_creds_inner(
    vault: &KeyringStore,
    env: &str,
    login: String,
    password: String,
) -> Result<(), String> {
    validate_env(env)?;
    set_vault_secret(vault, "hal", env, "login", &login)?;
    set_vault_secret(vault, "hal", env, "password", &password)
}

pub(crate) fn save_datacite_creds_inner(
    vault: &KeyringStore,
    env: &str,
    repository_id: String,
    password: String,
) -> Result<(), String> {
    validate_env(env)?;
    set_vault_secret(vault, "datacite", env, "repository_id", &repository_id)?;
    set_vault_secret(vault, "datacite", env, "password", &password)
}

pub(crate) fn get_hal_creds_inner(
    vault: &KeyringStore,
    env: &str,
) -> Result<Option<HalCredentialInfo>, String> {
    validate_env(env)?;
    let login = get_optional_vault_secret(vault, "hal", env, "login")?;
    Ok(login.map(|login| HalCredentialInfo { login }))
}

pub(crate) fn get_datacite_creds_inner(
    vault: &KeyringStore,
    env: &str,
) -> Result<Option<DataciteCredentialInfo>, String> {
    validate_env(env)?;
    let repository_id = get_optional_vault_secret(vault, "datacite", env, "repository_id")?;
    Ok(repository_id.map(|repository_id| DataciteCredentialInfo { repository_id }))
}

pub(crate) fn delete_creds_inner(vault: &KeyringStore, api: &str, env: &str) -> Result<(), String> {
    validate_env(env)?;
    let fields: &[&str] = match api {
        "hal" => &["login", "password"],
        "datacite" => &["repository_id", "password"],
        other => return Err(format!("unknown api: {other}")),
    };
    for field in fields {
        if let Ok(entry) = vault.build(SERVICE, &store_key(api, env, field), None) {
            let _ = entry.delete_credential();
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn save_hal_credentials(
    store: State<'_, CredentialStore>,
    env: String,
    login: String,
    password: String,
) -> Result<(), String> {
    let vault = store.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        save_hal_creds_inner(vault.as_ref(), &env, login, password)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn save_datacite_credentials(
    store: State<'_, CredentialStore>,
    env: String,
    repository_id: String,
    password: String,
) -> Result<(), String> {
    let vault = store.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        save_datacite_creds_inner(vault.as_ref(), &env, repository_id, password)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Returns the HAL login (non-sensitive) if credentials are stored, None otherwise.
/// The password never leaves the backend.
#[tauri::command]
pub async fn get_hal_credentials(
    store: State<'_, CredentialStore>,
    env: String,
) -> Result<Option<HalCredentialInfo>, String> {
    let vault = store.0.clone();
    tauri::async_runtime::spawn_blocking(move || get_hal_creds_inner(vault.as_ref(), &env))
        .await
        .map_err(|e| e.to_string())?
}

/// Returns the DataCite repository_id (non-sensitive) if credentials are stored, None otherwise.
/// The password never leaves the backend.
#[tauri::command]
pub async fn get_datacite_credentials(
    store: State<'_, CredentialStore>,
    env: String,
) -> Result<Option<DataciteCredentialInfo>, String> {
    let vault = store.0.clone();
    tauri::async_runtime::spawn_blocking(move || get_datacite_creds_inner(vault.as_ref(), &env))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn delete_credentials(
    store: State<'_, CredentialStore>,
    api: String,
    env: String,
) -> Result<(), String> {
    let vault = store.0.clone();
    tauri::async_runtime::spawn_blocking(move || delete_creds_inner(vault.as_ref(), &api, &env))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_vault() -> Arc<KeyringStore> {
        keyring_core::mock::Store::new().expect("failed to create mock credential store")
    }

    #[test]
    fn test_store_key_format() {
        assert_eq!(store_key("hal", "prod", "login"), "hal_prod_login");
        assert_eq!(
            store_key("datacite", "test", "repository_id"),
            "datacite_test_repository_id"
        );
    }

    #[test]
    fn test_save_and_get_hal_credentials_prod() {
        let vault = setup_vault();
        save_hal_creds_inner(vault.as_ref(), "prod", "alice".into(), "secret".into()).unwrap();
        let result = get_hal_creds_inner(vault.as_ref(), "prod").unwrap();
        assert_eq!(
            result,
            Some(HalCredentialInfo {
                login: "alice".into()
            })
        );
    }

    #[test]
    fn test_save_and_get_hal_credentials_test_env() {
        let vault = setup_vault();
        save_hal_creds_inner(vault.as_ref(), "test", "bob".into(), "pw".into()).unwrap();
        let result = get_hal_creds_inner(vault.as_ref(), "test").unwrap();
        assert_eq!(
            result,
            Some(HalCredentialInfo {
                login: "bob".into()
            })
        );
    }

    #[test]
    fn test_get_hal_credentials_when_empty() {
        let vault = setup_vault();
        let result = get_hal_creds_inner(vault.as_ref(), "prod").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_save_hal_overwrites_existing() {
        let vault = setup_vault();
        save_hal_creds_inner(vault.as_ref(), "prod", "alice".into(), "pw1".into()).unwrap();
        save_hal_creds_inner(vault.as_ref(), "prod", "bob".into(), "pw2".into()).unwrap();
        let result = get_hal_creds_inner(vault.as_ref(), "prod").unwrap();
        assert_eq!(
            result,
            Some(HalCredentialInfo {
                login: "bob".into()
            })
        );
    }

    #[test]
    fn test_save_and_get_datacite_credentials() {
        let vault = setup_vault();
        save_datacite_creds_inner(vault.as_ref(), "prod", "10.17616/R3".into(), "pw".into()).unwrap();
        let result = get_datacite_creds_inner(vault.as_ref(), "prod").unwrap();
        assert_eq!(
            result,
            Some(DataciteCredentialInfo {
                repository_id: "10.17616/R3".into()
            })
        );
    }

    #[test]
    fn test_get_datacite_credentials_when_empty() {
        let vault = setup_vault();
        let result = get_datacite_creds_inner(vault.as_ref(), "prod").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_delete_hal_credentials() {
        let vault = setup_vault();
        save_hal_creds_inner(vault.as_ref(), "prod", "alice".into(), "secret".into()).unwrap();
        delete_creds_inner(vault.as_ref(), "hal", "prod").unwrap();
        let result = get_hal_creds_inner(vault.as_ref(), "prod").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_delete_datacite_credentials() {
        let vault = setup_vault();
        save_datacite_creds_inner(vault.as_ref(), "prod", "10.17616/R3".into(), "pw".into()).unwrap();
        delete_creds_inner(vault.as_ref(), "datacite", "prod").unwrap();
        let result = get_datacite_creds_inner(vault.as_ref(), "prod").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_delete_unknown_api_returns_error() {
        let vault = setup_vault();
        let err = delete_creds_inner(vault.as_ref(), "unknown", "prod").unwrap_err();
        assert!(err.contains("unknown api: unknown"));
    }

    #[test]
    fn test_unknown_env_returns_error() {
        let vault = setup_vault();
        let err = save_hal_creds_inner(vault.as_ref(), "staging", "alice".into(), "pw".into()).unwrap_err();
        assert!(err.contains("unknown env: staging"));
    }

    /// The OS-level persistence guarantee of each real backend (Windows Credential
    /// Manager, the Secret Service) can't be exercised by a unit test, since the mock
    /// store is memory-only. This instead checks that reads never rely on caching: every
    /// call builds a fresh `Entry` from the shared store, so a value saved through one
    /// entry must be visible through another.
    #[test]
    fn test_read_does_not_depend_on_entry_caching() {
        let vault = setup_vault();
        save_hal_creds_inner(vault.as_ref(), "prod", "alice".into(), "secret".into()).unwrap();
        let result = get_hal_creds_inner(vault.as_ref(), "prod").unwrap();
        assert_eq!(
            result,
            Some(HalCredentialInfo {
                login: "alice".into()
            })
        );
    }

    #[test]
    fn test_prod_and_test_envs_are_independent() {
        let vault = setup_vault();
        save_hal_creds_inner(vault.as_ref(), "prod", "alice".into(), "pw".into()).unwrap();
        save_hal_creds_inner(vault.as_ref(), "test", "bob".into(), "pw".into()).unwrap();
        assert_eq!(
            get_hal_creds_inner(vault.as_ref(), "prod").unwrap(),
            Some(HalCredentialInfo {
                login: "alice".into()
            })
        );
        assert_eq!(
            get_hal_creds_inner(vault.as_ref(), "test").unwrap(),
            Some(HalCredentialInfo {
                login: "bob".into()
            })
        );
    }
}
