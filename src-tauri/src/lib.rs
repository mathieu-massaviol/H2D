mod credentials;
mod datacite_service;
mod hal_service;
mod utils;

use credentials::CredentialStore;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_stronghold::stronghold::Stronghold;

const CLIENT_NAME: &[u8] = b"hal2datacite";

fn derive_vault_key() -> Vec<u8> {
    use sha2::{Digest, Sha256};
    let machine_id =
        machine_uid::get().unwrap_or_else(|_| "hal2datacite-default".to_string());
    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    hasher.update(b"hal2datacite_v1");
    hasher.finalize().to_vec()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let key = derive_vault_key();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let vault_path = app
                .path()
                .app_data_dir()
                .expect("could not resolve app data dir")
                .join("credentials.hold");
            let vault =
                Stronghold::new(&vault_path, key).expect("failed to initialize vault");
            if vault.load_client(CLIENT_NAME).is_err() {
                vault
                    .create_client(CLIENT_NAME)
                    .expect("failed to create stronghold client");
                vault.save().expect("failed to save vault");
            }
            app.manage(CredentialStore(Arc::new(Mutex::new(vault))));
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
