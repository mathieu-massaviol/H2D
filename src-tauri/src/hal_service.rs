use crate::credentials::{get_hal_creds_inner, get_vault_secret, validate_env, CredentialStore};
use quick_xml::{Reader, Writer};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tauri::State;

// ── URL de base selon l'environnement ──────────────────────────────────────

fn hal_base_url(env: &str) -> &'static str {
    match env {
        "test" => "https://api-preprod.archives-ouvertes.fr",
        _ => "https://api.archives-ouvertes.fr",
    }
}

/// Base URL for the HAL web frontend (used to fetch TEI documents).
fn hal_web_url(env: &str) -> &'static str {
    match env {
        "test" => "https://hal.halpreprod.archives-ouvertes.fr",
        _ => "https://hal.science",
    }
}

// ── Normalisation de l'identifiant HAL ────────────────────────────────────

/// Accepte un halId (hal-XXXXXXX) ou une URL HAL complète.
/// Retourne le halId sans suffixe de version (ex: "hal-03720513").
pub(crate) fn normalize_hal_id(input: &str) -> String {
    let trimmed = input.trim();

    // Extraire le segment de chemin contenant le halId depuis une URL
    let id_segment = if trimmed.starts_with("http") {
        trimmed
            .trim_end_matches('/')
            .split('/')
            .find(|s| s.starts_with("hal-") || s.starts_with("halshs-") || s.starts_with("hal-shs-"))
            .unwrap_or(trimmed)
    } else {
        trimmed
    };

    // Supprimer le suffixe de version (v1, v2, ...)
    let id = if let Some(pos) = id_segment.rfind('v') {
        let (base, suffix) = id_segment.split_at(pos);
        if suffix[1..].chars().all(|c| c.is_ascii_digit()) {
            base
        } else {
            id_segment
        }
    } else {
        id_segment
    };

    id.to_string()
}

// ── Structures de désérialisation de la réponse HAL ───────────────────────

#[derive(Deserialize)]
struct HalApiResponse {
    response: HalResponseBody,
}

#[derive(Deserialize)]
struct HalResponseBody {
    docs: Vec<HalDoc>,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct HalDoc {
    title_s: Option<Vec<String>>,
    authFullName_s: Option<Vec<String>>,
    authFirstName_s: Option<Vec<String>>,
    authLastName_s: Option<Vec<String>>,
    authORCIDIdExt_s: Option<Vec<String>>,
    publicationDate_s: Option<String>,
    docType_s: Option<String>,
    abstract_s: Option<Vec<String>>,
    keyword_s: Option<Vec<String>>,
    language_s: Option<Vec<String>>,
    doiId_s: Option<String>,
    uri_s: Option<String>,
    licence_s: Option<String>,
    authIdHasStructure_fs: Option<Vec<String>>,
    anrProjectTitle_s: Option<Vec<String>>,
    anrProjectReference_s: Option<Vec<String>>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

// ── Structures retournées au frontend ─────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct HalAuthor {
    pub full_name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub orcid: Option<String>,
    pub affiliations: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HalMetadata {
    pub hal_id: String,
    pub title: Vec<String>,
    pub authors: Vec<HalAuthor>,
    pub publication_date: Option<String>,
    pub doc_type: Option<String>,
    pub abstracts: Vec<String>,
    pub keywords: Vec<String>,
    pub languages: Vec<String>,
    pub existing_doi: Option<String>,
    pub uri: Option<String>,
    pub license: Option<String>,
    pub titles_by_lang: HashMap<String, Vec<String>>,
    pub abstracts_by_lang: HashMap<String, Vec<String>>,
    pub keywords_by_lang: HashMap<String, Vec<String>>,
    pub anr_project_titles: Vec<String>,
    pub anr_project_references: Vec<String>,
}

// ── Champs demandés à l'API HAL ───────────────────────────────────────────

const HAL_FIELDS: &str = "halId_s,title_s,*_title_s,\
    authFullName_s,authFirstName_s,authLastName_s,authORCIDIdExt_s,authIdHasStructure_fs,\
    publicationDate_s,docType_s,abstract_s,*_abstract_s,keyword_s,*_keyword_s,language_s,\
    doiId_s,uri_s,licence_s,\
    anrProjectTitle_s,anrProjectReference_s";

// ── Mapping HalDoc → HalMetadata ──────────────────────────────────────────

/// Parse `authIdHasStructure_fs` entries into a map of author name → unique structure names.
/// Entry format: `{authorId}-{qualityIdx}_FacetSep_{authorName}_JoinSep_{structId}_FacetSep_{structName}`
fn parse_affiliations_by_author(entries: Vec<String>) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, HashSet<String>> = HashMap::new();
    for entry in entries {
        let Some((author_part, struct_part)) = entry.split_once("_JoinSep_") else {
            continue;
        };
        let Some((_, author_name)) = author_part.split_once("_FacetSep_") else {
            continue;
        };
        let Some((_, struct_name)) = struct_part.split_once("_FacetSep_") else {
            continue;
        };
        map.entry(author_name.to_string())
            .or_default()
            .insert(struct_name.to_string());
    }
    map.into_iter()
        .map(|(author, affiliations)| (author, affiliations.into_iter().collect()))
        .collect()
}

fn build_metadata(hal_id: String, doc: HalDoc) -> HalMetadata {
    let full_names = doc.authFullName_s.unwrap_or_default();
    let first_names = doc.authFirstName_s.unwrap_or_default();
    let last_names = doc.authLastName_s.unwrap_or_default();
    let orcids = doc.authORCIDIdExt_s.unwrap_or_default();
    let affiliations_by_author =
        parse_affiliations_by_author(doc.authIdHasStructure_fs.unwrap_or_default());

    let mut titles_by_lang: HashMap<String, Vec<String>> = HashMap::new();
    let mut abstracts_by_lang: HashMap<String, Vec<String>> = HashMap::new();
    let mut keywords_by_lang: HashMap<String, Vec<String>> = HashMap::new();

    for (key, val) in doc.extra {
        if let Some(lang) = key.strip_suffix("_title_s") {
            if lang.len() == 2 && lang.chars().all(|c| c.is_ascii_lowercase()) {
                if let Ok(titles) = serde_json::from_value::<Vec<String>>(val) {
                    titles_by_lang.insert(lang.to_string(), titles);
                }
            }
        } else if let Some(lang) = key.strip_suffix("_abstract_s") {
            if lang.len() == 2 && lang.chars().all(|c| c.is_ascii_lowercase()) {
                if let Ok(abstracts) = serde_json::from_value::<Vec<String>>(val) {
                    abstracts_by_lang.insert(lang.to_string(), abstracts);
                }
            }
        } else if let Some(lang) = key.strip_suffix("_keyword_s") {
            if lang.len() == 2 && lang.chars().all(|c| c.is_ascii_lowercase()) {
                if let Ok(kws) = serde_json::from_value::<Vec<String>>(val) {
                    keywords_by_lang.insert(lang.to_string(), kws);
                }
            }
        }
    }

    let authors: Vec<HalAuthor> = full_names
        .into_iter()
        .enumerate()
        .map(|(i, full_name)| {
            let affiliations = affiliations_by_author
                .get(&full_name)
                .cloned()
                .unwrap_or_default();
            HalAuthor {
                full_name,
                first_name: first_names.get(i).cloned(),
                last_name: last_names.get(i).cloned(),
                orcid: orcids.get(i).cloned(),
                affiliations,
            }
        })
        .collect();

    HalMetadata {
        hal_id,
        title: doc.title_s.unwrap_or_default(),
        authors,
        publication_date: doc.publicationDate_s,
        doc_type: doc.docType_s,
        abstracts: doc.abstract_s.unwrap_or_default(),
        keywords: doc.keyword_s.unwrap_or_default(),
        languages: doc.language_s.unwrap_or_default(),
        existing_doi: doc.doiId_s,
        uri: doc.uri_s,
        license: doc.licence_s,
        titles_by_lang,
        abstracts_by_lang,
        keywords_by_lang,
        anr_project_titles: doc.anrProjectTitle_s.unwrap_or_default(),
        anr_project_references: doc.anrProjectReference_s.unwrap_or_default(),
    }
}

// ── Commandes Tauri ───────────────────────────────────────────────────────

/// Récupère les métadonnées d'une fiche HAL.
/// Accepte un halId (hal-XXXXXXX) ou une URL HAL complète.
#[tauri::command]
pub async fn get_hal_metadata(hal_id: String, env: String) -> Result<HalMetadata, String> {
    validate_env(&env)?;
    let normalized = normalize_hal_id(&hal_id);
    let url = format!(
        "{}/search/?q=halId_s:{}&fl={}",
        hal_base_url(&env),
        normalized,
        HAL_FIELDS
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HAL API request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("HAL API returned status {}", response.status()));
    }

    let api_response: HalApiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse HAL API response: {e}"))?;

    let doc = api_response
        .response
        .docs
        .into_iter()
        .next()
        .ok_or_else(|| format!("HAL record not found: {normalized}"))?;

    Ok(build_metadata(normalized, doc))
}

// ── Manipulation du XML TEI pour l'insertion du DOI ──────────────────────

/// Insère `<idno type="doi">{doi}</idno>` dans `<biblStruct>` du TEI HAL.
/// Reproduit la logique du script Python de référence :
/// - avant le premier `<ref>` direct de `<biblStruct>` s'il existe,
/// - sinon en dernier enfant avant `</biblStruct>`.
fn insert_doi_in_tei(tei_xml: &str, doi: &str) -> Result<String, String> {
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(tei_xml);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Vec::new());

    let mut in_bib_struct = false;
    let mut doi_inserted = false;
    let mut depth_in_bib_struct: u32 = 0;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let local = e.local_name();
                let name_str: &str = local.as_ref();

                if !in_bib_struct && name_str == "biblStruct" {
                    in_bib_struct = true;
                    depth_in_bib_struct = 0;
                    writer.write_event(Event::Start(e)).map_err(|e| e.to_string())?;
                } else if in_bib_struct {
                    depth_in_bib_struct += 1;
                    // Insérer le DOI avant le premier <ref> direct de <biblStruct>
                    if depth_in_bib_struct == 1 && name_str == "ref" && !doi_inserted {
                        write_idno_doi(&mut writer, doi).map_err(|e| e.to_string())?;
                        doi_inserted = true;
                    }
                    writer.write_event(Event::Start(e)).map_err(|e| e.to_string())?;
                } else {
                    writer.write_event(Event::Start(e)).map_err(|e| e.to_string())?;
                }
            }
            Ok(Event::End(e)) => {
                let local = e.local_name();
                let name_str: &str = local.as_ref();

                if in_bib_struct && name_str == "biblStruct" && depth_in_bib_struct == 0 {
                    // Fermeture de <biblStruct> : insérer le DOI s'il ne l'a pas encore été
                    if !doi_inserted {
                        write_idno_doi(&mut writer, doi).map_err(|e| e.to_string())?;
                    }
                    in_bib_struct = false;
                    writer.write_event(Event::End(e)).map_err(|e| e.to_string())?;
                } else if in_bib_struct && depth_in_bib_struct > 0 {
                    depth_in_bib_struct -= 1;
                    writer.write_event(Event::End(e)).map_err(|e| e.to_string())?;
                } else {
                    writer.write_event(Event::End(e)).map_err(|e| e.to_string())?;
                }
            }
            Ok(Event::Empty(e)) => {
                let local = e.local_name();
                let name_str: &str = local.as_ref();
                // Cas d'un <ref/> auto-fermant direct dans <biblStruct>
                if in_bib_struct && depth_in_bib_struct == 0 && name_str == "ref" && !doi_inserted {
                    write_idno_doi(&mut writer, doi).map_err(|e| e.to_string())?;
                    doi_inserted = true;
                }
                writer.write_event(Event::Empty(e)).map_err(|e| e.to_string())?;
            }
            Ok(Event::Eof) => break,
            Ok(e) => writer.write_event(e).map_err(|e| e.to_string())?,
            Err(e) => return Err(format!("XML parse error: {e}")),
        }
    }

    String::from_utf8(writer.into_inner()).map_err(|e| e.to_string())
}

fn write_idno_doi(writer: &mut Writer<Vec<u8>>, doi: &str) -> Result<(), quick_xml::Error> {
    use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
    let mut start = BytesStart::new("idno");
    start.push_attribute(("type", "doi"));
    writer.write_event(Event::Start(start))?;
    writer.write_event(Event::Text(BytesText::new(doi)))?;
    writer.write_event(Event::End(BytesEnd::new("idno")))?;
    Ok(())
}

// ── Commande Tauri ─────────────────────────────────────────────────────────

/// Ajoute un DOI à une fiche HAL via l'API SWORD2.
/// 1. Récupère le TEI complet de la fiche (GET /sword/{id}/tei).
/// 2. Insère <idno type="doi"> dans <biblStruct>.
/// 3. Renvoie le TEI modifié (PUT /sword/{id}).
#[tauri::command]
pub async fn add_doi_to_hal_record(
    store: State<'_, CredentialStore>,
    hal_id: String,
    doi: String,
    env: String,
) -> Result<(), String> {
    validate_env(&env)?;

    // Récupérer les credentials HAL depuis le vault
    let store_arc = store.0.clone();
    let env_clone = env.clone();
    let (login, password) = tauri::async_runtime::spawn_blocking(move || {
        let vault = store_arc.as_ref();
        let login = get_hal_creds_inner(vault, &env_clone)?
            .ok_or_else(|| "HAL credentials not configured".to_string())?
            .login;
        let password = get_vault_secret(vault, "hal", &env_clone, "password")?;
        Ok::<(String, String), String>((login, password))
    })
    .await
    .map_err(|e| e.to_string())??;

    let normalized = normalize_hal_id(&hal_id);
    let base = hal_base_url(&env);
    let client = reqwest::Client::new();

    // 1. Récupérer le TEI de la fiche HAL
    let tei_url = format!("{}/{}/tei", hal_web_url(&env), normalized);
    let tei_response = client
        .get(&tei_url)
        .send()
        .await
        .map_err(|e| format!("HAL TEI request failed: {e}"))?;

    if !tei_response.status().is_success() {
        let status = tei_response.status();
        let body = tei_response.text().await.unwrap_or_default();
        return Err(format!("HAL TEI GET {tei_url} returned status {status}: {body}"));
    }

    let tei_xml = tei_response
        .text()
        .await
        .map_err(|e| format!("Failed to read HAL TEI response: {e}"))?;

    // 2. Insérer le DOI dans le XML TEI
    let modified_tei = insert_doi_in_tei(&tei_xml, &doi)?;

    // 3. PUT le TEI modifié via SWORD
    let sword_url = format!("{}/sword/{}", base, normalized);
    let response = client
        .put(&sword_url)
        .basic_auth(&login, Some(&password))
        .header("Content-Type", "text/xml")
        .header("Packaging", "http://purl.org/net/sword-types/AOfr")
        .body(modified_tei)
        .send()
        .await
        .map_err(|e| format!("HAL SWORD request failed: {e}"))?;

    match response.status().as_u16() {
        200 | 201 => Ok(()),
        400 => {
            let body = response.text().await.unwrap_or_default();
            if body.contains("Paper Id can not be updated") {
                Err("HAL_NO_WRITE_ACCESS".to_string())
            } else {
                Err(format!("HAL SWORD returned status 400: {body}"))
            }
        }
        401 => Err("HAL authentication failed: invalid credentials".to_string()),
        403 => Err("HAL authorization failed: insufficient permissions".to_string()),
        404 => Err(format!("HAL record not found: {normalized}")),
        status => {
            let body = response.text().await.unwrap_or_default();
            Err(format!("HAL SWORD returned status {status}: {body}"))
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_plain_hal_id() {
        assert_eq!(normalize_hal_id("hal-03720513"), "hal-03720513");
    }

    #[test]
    fn test_normalize_hal_id_with_version() {
        assert_eq!(normalize_hal_id("hal-03720513v1"), "hal-03720513");
        assert_eq!(normalize_hal_id("hal-03720513v12"), "hal-03720513");
    }

    #[test]
    fn test_normalize_hal_science_url() {
        assert_eq!(
            normalize_hal_id("https://hal.science/hal-03720513v1"),
            "hal-03720513"
        );
    }

    #[test]
    fn test_normalize_hal_archives_url() {
        assert_eq!(
            normalize_hal_id("https://hal.archives-ouvertes.fr/hal-03720513"),
            "hal-03720513"
        );
    }

    #[test]
    fn test_normalize_trims_whitespace() {
        assert_eq!(normalize_hal_id("  hal-03720513  "), "hal-03720513");
    }

    #[test]
    fn test_hal_base_url_prod() {
        assert_eq!(hal_base_url("prod"), "https://api.archives-ouvertes.fr");
    }

    #[test]
    fn test_hal_base_url_test() {
        assert_eq!(
            hal_base_url("test"),
            "https://api-preprod.archives-ouvertes.fr"
        );
    }
}
