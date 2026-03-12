use std::sync::{Arc, Mutex};

use tauri::State;
use tauri_plugin_stronghold::stronghold::Stronghold;

const CLIENT_NAME: &[u8] = b"hal2datacite";

pub struct CredentialStore(pub Arc<Mutex<Stronghold>>);

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

/// Reads a single secret from the vault. Returns `Err` if the key is absent or not valid UTF-8.
pub(crate) fn get_vault_secret(
    vault: &Stronghold,
    api: &str,
    env: &str,
    field: &str,
) -> Result<String, String> {
    let client = vault.get_client(CLIENT_NAME).map_err(|e| e.to_string())?;
    client
        .store()
        .get(store_key(api, env, field).as_ref())
        .map_err(|e| e.to_string())?
        .and_then(|b| String::from_utf8(b).ok())
        .ok_or_else(|| format!("{api} {field} not configured"))
}

pub(crate) fn save_hal_creds_inner(
    vault: &Stronghold,
    env: &str,
    login: String,
    password: String,
) -> Result<(), String> {
    validate_env(env)?;
    let client = vault.get_client(CLIENT_NAME).map_err(|e| e.to_string())?;
    let s = client.store();
    s.insert(
        store_key("hal", env, "login").into_bytes(),
        login.into_bytes(),
        None,
    )
    .map_err(|e| e.to_string())?;
    s.insert(
        store_key("hal", env, "password").into_bytes(),
        password.into_bytes(),
        None,
    )
    .map_err(|e| e.to_string())?;
    vault.save().map_err(|e| e.to_string())
}

pub(crate) fn save_datacite_creds_inner(
    vault: &Stronghold,
    env: &str,
    repository_id: String,
    password: String,
) -> Result<(), String> {
    validate_env(env)?;
    let client = vault.get_client(CLIENT_NAME).map_err(|e| e.to_string())?;
    let s = client.store();
    s.insert(
        store_key("datacite", env, "repository_id").into_bytes(),
        repository_id.into_bytes(),
        None,
    )
    .map_err(|e| e.to_string())?;
    s.insert(
        store_key("datacite", env, "password").into_bytes(),
        password.into_bytes(),
        None,
    )
    .map_err(|e| e.to_string())?;
    vault.save().map_err(|e| e.to_string())
}

pub(crate) fn get_hal_creds_inner(
    vault: &Stronghold,
    env: &str,
) -> Result<Option<HalCredentialInfo>, String> {
    validate_env(env)?;
    let client = vault.get_client(CLIENT_NAME).map_err(|e| e.to_string())?;
    let s = client.store();
    let login = s
        .get(store_key("hal", env, "login").as_ref())
        .map_err(|e| e.to_string())?
        .and_then(|b| String::from_utf8(b).ok());
    Ok(login.map(|login| HalCredentialInfo { login }))
}

pub(crate) fn get_datacite_creds_inner(
    vault: &Stronghold,
    env: &str,
) -> Result<Option<DataciteCredentialInfo>, String> {
    validate_env(env)?;
    let client = vault.get_client(CLIENT_NAME).map_err(|e| e.to_string())?;
    let s = client.store();
    let repository_id = s
        .get(store_key("datacite", env, "repository_id").as_ref())
        .map_err(|e| e.to_string())?
        .and_then(|b| String::from_utf8(b).ok());
    Ok(repository_id.map(|repository_id| DataciteCredentialInfo { repository_id }))
}

pub(crate) fn delete_creds_inner(
    vault: &Stronghold,
    api: &str,
    env: &str,
) -> Result<(), String> {
    validate_env(env)?;
    let client = vault.get_client(CLIENT_NAME).map_err(|e| e.to_string())?;
    let s = client.store();
    let fields: &[&str] = match api {
        "hal" => &["login", "password"],
        "datacite" => &["repository_id", "password"],
        other => return Err(format!("unknown api: {other}")),
    };
    for field in fields {
        let _ = s.delete(store_key(api, env, field).as_ref());
    }
    vault.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_hal_credentials(
    store: State<'_, CredentialStore>,
    env: String,
    login: String,
    password: String,
) -> Result<(), String> {
    let store = store.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let vault = store.lock().map_err(|e| e.to_string())?;
        save_hal_creds_inner(&vault, &env, login, password)
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
    let store = store.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let vault = store.lock().map_err(|e| e.to_string())?;
        save_datacite_creds_inner(&vault, &env, repository_id, password)
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
    let store = store.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let vault = store.lock().map_err(|e| e.to_string())?;
        get_hal_creds_inner(&vault, &env)
    })
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
    let store = store.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let vault = store.lock().map_err(|e| e.to_string())?;
        get_datacite_creds_inner(&vault, &env)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn delete_credentials(
    store: State<'_, CredentialStore>,
    api: String,
    env: String,
) -> Result<(), String> {
    let store = store.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let vault = store.lock().map_err(|e| e.to_string())?;
        delete_creds_inner(&vault, &api, &env)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_vault() -> (TempDir, Stronghold) {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path().join("test.hold");
        let key = vec![0u8; 32];
        let vault = Stronghold::new(&path, key).expect("failed to create vault");
        vault
            .create_client(CLIENT_NAME)
            .expect("failed to create client");
        vault.save().expect("failed to save vault");
        (dir, vault)
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
        let (_dir, vault) = setup_vault();
        save_hal_creds_inner(&vault, "prod", "alice".into(), "secret".into()).unwrap();
        let result = get_hal_creds_inner(&vault, "prod").unwrap();
        assert_eq!(
            result,
            Some(HalCredentialInfo {
                login: "alice".into()
            })
        );
    }

    #[test]
    fn test_save_and_get_hal_credentials_test_env() {
        let (_dir, vault) = setup_vault();
        save_hal_creds_inner(&vault, "test", "bob".into(), "pw".into()).unwrap();
        let result = get_hal_creds_inner(&vault, "test").unwrap();
        assert_eq!(
            result,
            Some(HalCredentialInfo {
                login: "bob".into()
            })
        );
    }

    #[test]
    fn test_get_hal_credentials_when_empty() {
        let (_dir, vault) = setup_vault();
        let result = get_hal_creds_inner(&vault, "prod").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_save_hal_overwrites_existing() {
        let (_dir, vault) = setup_vault();
        save_hal_creds_inner(&vault, "prod", "alice".into(), "pw1".into()).unwrap();
        save_hal_creds_inner(&vault, "prod", "bob".into(), "pw2".into()).unwrap();
        let result = get_hal_creds_inner(&vault, "prod").unwrap();
        assert_eq!(
            result,
            Some(HalCredentialInfo {
                login: "bob".into()
            })
        );
    }

    #[test]
    fn test_save_and_get_datacite_credentials() {
        let (_dir, vault) = setup_vault();
        save_datacite_creds_inner(&vault, "prod", "10.17616/R3".into(), "pw".into()).unwrap();
        let result = get_datacite_creds_inner(&vault, "prod").unwrap();
        assert_eq!(
            result,
            Some(DataciteCredentialInfo {
                repository_id: "10.17616/R3".into()
            })
        );
    }

    #[test]
    fn test_get_datacite_credentials_when_empty() {
        let (_dir, vault) = setup_vault();
        let result = get_datacite_creds_inner(&vault, "prod").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_delete_hal_credentials() {
        let (_dir, vault) = setup_vault();
        save_hal_creds_inner(&vault, "prod", "alice".into(), "secret".into()).unwrap();
        delete_creds_inner(&vault, "hal", "prod").unwrap();
        let result = get_hal_creds_inner(&vault, "prod").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_delete_datacite_credentials() {
        let (_dir, vault) = setup_vault();
        save_datacite_creds_inner(&vault, "prod", "10.17616/R3".into(), "pw".into()).unwrap();
        delete_creds_inner(&vault, "datacite", "prod").unwrap();
        let result = get_datacite_creds_inner(&vault, "prod").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_delete_unknown_api_returns_error() {
        let (_dir, vault) = setup_vault();
        let err = delete_creds_inner(&vault, "unknown", "prod").unwrap_err();
        assert!(err.contains("unknown api: unknown"));
    }

    #[test]
    fn test_unknown_env_returns_error() {
        let (_dir, vault) = setup_vault();
        let err = save_hal_creds_inner(&vault, "staging", "alice".into(), "pw".into()).unwrap_err();
        assert!(err.contains("unknown env: staging"));
    }

    #[test]
    fn test_persistence_across_restarts() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.hold");
        let key = vec![0u8; 32];

        // Session 1 : sauvegarder
        {
            let vault = Stronghold::new(&path, key.clone()).unwrap();
            vault.create_client(CLIENT_NAME).unwrap();
            vault.save().unwrap();
            save_hal_creds_inner(&vault, "prod", "alice".into(), "secret".into()).unwrap();
        }

        // Session 2 : charger depuis le fichier
        {
            let vault = Stronghold::new(&path, key.clone()).unwrap();
            vault.load_client(CLIENT_NAME).unwrap();
            let result = get_hal_creds_inner(&vault, "prod").unwrap();
            assert_eq!(result, Some(HalCredentialInfo { login: "alice".into() }));
        }
    }

    #[test]
    fn test_prod_and_test_envs_are_independent() {
        let (_dir, vault) = setup_vault();
        save_hal_creds_inner(&vault, "prod", "alice".into(), "pw".into()).unwrap();
        save_hal_creds_inner(&vault, "test", "bob".into(), "pw".into()).unwrap();
        assert_eq!(
            get_hal_creds_inner(&vault, "prod").unwrap(),
            Some(HalCredentialInfo {
                login: "alice".into()
            })
        );
        assert_eq!(
            get_hal_creds_inner(&vault, "test").unwrap(),
            Some(HalCredentialInfo {
                login: "bob".into()
            })
        );
    }
}
