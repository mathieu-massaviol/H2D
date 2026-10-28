mod credentials;
mod datacite_service;
mod hal_service;
mod utils;

use credentials::CredentialStore;
use tauri::Manager;

#[cfg(target_os = "windows")]
fn platform_credential_store() -> Result<std::sync::Arc<keyring_core::CredentialStore>, String> {
    windows_native_keyring_store::Store::new()
        .map(|store| store as std::sync::Arc<keyring_core::CredentialStore>)
        .map_err(|e| e.to_string())
}

#[cfg(target_os = "linux")]
fn platform_credential_store() -> Result<std::sync::Arc<keyring_core::CredentialStore>, String> {
    zbus_secret_service_keyring_store::Store::new()
        .map(|store| store as std::sync::Arc<keyring_core::CredentialStore>)
        .map_err(|e| e.to_string())
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn platform_credential_store() -> Result<std::sync::Arc<keyring_core::CredentialStore>, String> {
    compile_error!("no credential store backend is configured for this platform");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let store = platform_credential_store()
                .map_err(|e| format!("failed to initialize credential store: {e}"))?;
            app.manage(CredentialStore(store));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            credentials::save_hal_credentials,
            credentials::save_datacite_credentials,
            credentials::get_hal_credentials,
            credentials::get_datacite_credentials,
            credentials::delete_credentials,
            hal_service::get_hal_metadata,
            hal_service::add_doi_to_hal_record,
            datacite_service::map_hal_to_datacite,
            datacite_service::create_doi,
            datacite_service::get_datacite_prefixes,
            datacite_service::publish_doi,
            datacite_service::delete_doi,
            datacite_service::get_datacite_dois,
            utils::save_csv,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
