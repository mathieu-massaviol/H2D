use crate::credentials::{get_vault_secret, CredentialStore};
use crate::hal_service::HalMetadata;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tauri::State;

// ── Structures DataCite ────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataciteNameIdentifier {
    pub scheme_uri: String,
    pub name_identifier: String,
    pub name_identifier_scheme: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataciteAffiliation {
    pub name: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataciteCreator {
    pub name: String,
    pub name_type: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub affiliations: Option<Vec<DataciteAffiliation>>,
    pub name_identifiers: Option<Vec<DataciteNameIdentifier>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataciteTitle {
    pub title: String,
    pub lang: Option<String>,
    pub title_type: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataciteDescription {
    pub description: String,
    pub description_type: String,
    pub lang: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataciteRights {
    pub rights: Option<String>,
    pub rights_uri: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ResourceTypeGeneral {
    Audiovisual,
    Award,
    Book,
    BookChapter,
    Collection,
    ComputationalNotebook,
    ConferencePaper,
    ConferenceProceeding,
    DataPaper,
    Dataset,
    Dissertation,
    Event,
    Image,
    InteractiveResource,
    Instrument,
    Journal,
    JournalArticle,
    Model,
    OutputManagementPlan,
    PeerReview,
    PhysicalObject,
    Poster,
    Preprint,
    Presentation,
    Project,
    Report,
    Service,
    Software,
    Sound,
    Standard,
    StudyRegistration,
    Text,
    Workflow,
    Other,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DataCiteFounderIdentifierType {
    #[serde(rename = "Crossref Funder ID")]
    CrossrefFunderID,
    GRID,
    ISNI,
    ROR,
    Other,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataciteTypes {
    pub resource_type_general: ResourceTypeGeneral,
    pub resource_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DatacitePublisher {
    pub name: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataciteSubject {
    pub subject: String,
    pub scheme_uri: Option<String>,
    pub value_uri: Option<String>,
    pub subject_scheme: Option<String>,
    pub lang: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataciteAlternateIdentifier {
    pub alternate_identifier: String,
    pub alternate_identifier_type: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataciteFundingReference {
    pub award_uri: Option<String>,
    pub award_title: Option<String>,
    pub funder_name: String,
    pub award_number: Option<String>,
    pub funder_identifier: Option<String>,
    pub funder_identifier_type: Option<DataCiteFounderIdentifierType>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataciteMetadata {
    pub creators: Vec<DataciteCreator>,
    pub titles: Vec<DataciteTitle>,
    pub publisher: DatacitePublisher,
    pub publication_year: u32,
    pub subjects: Option<Vec<DataciteSubject>>,
    pub language: Option<String>,
    pub types: DataciteTypes,
    pub alternate_identifiers: Option<Vec<DataciteAlternateIdentifier>>,
    pub funding_references: Option<Vec<DataciteFundingReference>>,
    pub descriptions: Option<Vec<DataciteDescription>>,
    pub rights_list: Option<Vec<DataciteRights>>,
    pub url: String,
}

// ── Mapping ────────────────────────────────────────────────────────────────

fn map_doc_type(doc_type: Option<&str>) -> ResourceTypeGeneral {
    match doc_type {
        Some("ART") => ResourceTypeGeneral::JournalArticle,
        Some("COMM") => ResourceTypeGeneral::ConferencePaper,
        Some("THESE") => ResourceTypeGeneral::Dissertation,
        Some("HDR") => ResourceTypeGeneral::Dissertation,
        Some("OUV") => ResourceTypeGeneral::Book,
        Some("COUV") => ResourceTypeGeneral::BookChapter,
        Some("DOUV") => ResourceTypeGeneral::BookChapter,
        Some("REPORT") => ResourceTypeGeneral::Report,
        Some("PREPRINT") => ResourceTypeGeneral::Preprint,
        Some("POSTER") => ResourceTypeGeneral::Poster,
        Some("PRESCONF") => ResourceTypeGeneral::Presentation,
        Some("VIDEO") => ResourceTypeGeneral::Audiovisual,
        Some("SON") => ResourceTypeGeneral::Sound,
        Some("IMG") => ResourceTypeGeneral::Image,
        Some("SOFTWARE") => ResourceTypeGeneral::Software,
        _ => ResourceTypeGeneral::Other,
    }
}

/// Transforme un `HalMetadata` en `DataciteMetadata`.
/// Le prefix est passé par le frontend (choisi par l'utilisateur).
pub(crate) fn map(hal_metadata: HalMetadata) -> Result<DataciteMetadata, String> {
    // publication_year: required
    let publication_year = hal_metadata
        .publication_date
        .as_deref()
        .and_then(|d| d.get(..4))
        .and_then(|y| y.parse::<u32>().ok())
        .ok_or_else(|| "publication_year manquant".to_string())?;

    // url: required
    let url = hal_metadata
        .uri
        .ok_or_else(|| "url HAL manquante".to_string())?;

    // titles: prefer titles_by_lang, fallback to title
    let titles: Vec<DataciteTitle> = if hal_metadata.titles_by_lang.is_empty() {
        hal_metadata
            .title
            .into_iter()
            .map(|t| DataciteTitle {
                title: t,
                lang: None,
                title_type: None,
            })
            .collect()
    } else {
        let mut result = Vec::new();
        for (lang, lang_titles) in &hal_metadata.titles_by_lang {
            for t in lang_titles {
                result.push(DataciteTitle {
                    title: t.clone(),
                    lang: Some(lang.clone()),
                    title_type: None,
                });
            }
        }
        result
    };

    // creators
    let creators: Vec<DataciteCreator> = hal_metadata
        .authors
        .into_iter()
        .map(|author| {
            let name_identifiers = author.orcid.map(|orcid| {
                vec![DataciteNameIdentifier {
                    name_identifier_scheme: "ORCID".to_string(),
                    scheme_uri: "https://orcid.org".to_string(),
                    name_identifier: orcid,
                }]
            });
            let affiliations = if author.affiliations.is_empty() {
                None
            } else {
                Some(
                    author
                        .affiliations
                        .into_iter()
                        .map(|name| DataciteAffiliation { name })
                        .collect(),
                )
            };
            DataciteCreator {
                name: author.full_name,
                name_type: "Personal".to_string(),
                given_name: author.first_name,
                family_name: author.last_name,
                affiliations,
                name_identifiers,
            }
        })
        .collect();

    // types
    let types = DataciteTypes {
        resource_type_general: map_doc_type(hal_metadata.doc_type.as_deref()),
        resource_type: hal_metadata.doc_type.clone(),
    };

    // subjects: préférer keywords_by_lang (avec langue), fallback sur keywords sans langue
    let subjects = if !hal_metadata.keywords_by_lang.is_empty() {
        let mut result: Vec<DataciteSubject> = Vec::new();
        for (lang, kws) in &hal_metadata.keywords_by_lang {
            for kw in kws {
                result.push(DataciteSubject {
                    subject: kw.clone(),
                    scheme_uri: None,
                    value_uri: None,
                    subject_scheme: None,
                    lang: Some(lang.clone()),
                });
            }
        }
        Some(result)
    } else if !hal_metadata.keywords.is_empty() {
        Some(
            hal_metadata
                .keywords
                .into_iter()
                .map(|kw| DataciteSubject {
                    subject: kw,
                    scheme_uri: None,
                    value_uri: None,
                    subject_scheme: None,
                    lang: None,
                })
                .collect(),
        )
    } else {
        None
    };

    // language
    let language = hal_metadata.languages.into_iter().next();

    // descriptions: préférer abstracts_by_lang (avec langue), fallback sur abstracts avec
    // la langue du document (language_s), ou sans langue si inconnue
    let descriptions = if !hal_metadata.abstracts_by_lang.is_empty() {
        let mut result: Vec<DataciteDescription> = Vec::new();
        for (lang, abstracts) in &hal_metadata.abstracts_by_lang {
            for abs in abstracts {
                result.push(DataciteDescription {
                    description: abs.clone(),
                    description_type: "Abstract".to_string(),
                    lang: Some(lang.clone()),
                });
            }
        }
        Some(result)
    } else if !hal_metadata.abstracts.is_empty() {
        let lang = language.clone();
        Some(
            hal_metadata
                .abstracts
                .into_iter()
                .map(|abs| DataciteDescription {
                    description: abs,
                    description_type: "Abstract".to_string(),
                    lang: lang.clone(),
                })
                .collect(),
        )
    } else {
        None
    };

    // rights_list
    let rights_list = hal_metadata.license.map(|lic| {
        vec![DataciteRights {
            rights: None,
            rights_uri: Some(lic),
        }]
    });

    // alternate_identifiers
    let alternate_identifiers = Some(vec![DataciteAlternateIdentifier {
        alternate_identifier: hal_metadata.hal_id,
        alternate_identifier_type: "HAL identifier".to_string(),
    }]);

    Ok(DataciteMetadata {
        creators,
        titles,
        publisher: DatacitePublisher {
            name: "HAL".to_string(),
        },
        publication_year,
        subjects,
        language,
        types,
        alternate_identifiers,
        funding_references: {
            let refs: Vec<DataciteFundingReference> = hal_metadata
                .anr_project_titles
                .into_iter()
                .zip(hal_metadata.anr_project_references)
                .map(|(title, reference)| DataciteFundingReference {
                    funder_name: "Agence Nationale de la Recherche".to_string(),
                    funder_identifier: Some(
                        "https://doi.org/10.13039/501100001665".to_string(),
                    ),
                    funder_identifier_type: Some(DataCiteFounderIdentifierType::CrossrefFunderID),
                    award_number: Some(reference),
                    award_title: Some(title),
                    award_uri: None,
                })
                .collect();
            if refs.is_empty() { None } else { Some(refs) }
        },
        descriptions,
        rights_list,
        url,
    })
}

// ── Création de DOI ────────────────────────────────────────────────────────

fn datacite_base_url(env: &str) -> &'static str {
    match env {
        "test" => "https://api.test.datacite.org",
        _ => "https://api.datacite.org",
    }
}

#[skip_serializing_none]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DataciteDoiAttributes {
    doi: Option<String>,
    prefix: Option<String>,
    event: Option<String>,
    #[serde(flatten)]
    metadata: DataciteMetadata,
}

#[derive(Serialize, Debug)]
struct DataciteDoiData {
    #[serde(rename = "type")]
    data_type: &'static str,
    attributes: DataciteDoiAttributes,
}

#[derive(Serialize, Debug)]
struct DataciteDoiRequest {
    data: DataciteDoiData,
}

#[derive(Deserialize, Debug)]
struct DataciteDoiResponseAttributes {
    doi: String,
}

#[derive(Deserialize, Debug)]
struct DataciteDoiResponseData {
    attributes: DataciteDoiResponseAttributes,
}

#[derive(Deserialize, Debug)]
struct DataciteDoiResponse {
    data: DataciteDoiResponseData,
}

/// Crée un DOI DataCite à partir des métadonnées HAL.
/// `publish`: `true` → DOI findable immédiatement, `false` → état draft
/// `suffix_prefix`: préfixe optionnel du suffixe du DOI (ex: "my-lab" → "10.XXXX/my-lab-4a9f82bc")
#[tauri::command]
pub async fn create_doi(
    store: State<'_, CredentialStore>,
    hal_metadata: HalMetadata,
    prefix: String,
    env: String,
    publish: bool,
    suffix_prefix: Option<String>,
) -> Result<String, String> {
    let (repository_id, password) = fetch_datacite_credentials(&store, env.clone()).await?;

    let metadata = map(hal_metadata)?;

    let client = reqwest::Client::new();

    let (doi_attr, prefix_attr) = if let Some(sp) = suffix_prefix.filter(|s| !s.is_empty()) {
        let mut candidate = None;
        for _ in 0..3 {
            let random = format!("{:08x}", rand::thread_rng().gen::<u32>());
            let full_doi = format!("{}/{}-{}", prefix, sp, random);
            if !doi_exists(&client, &full_doi, &repository_id, &password, &env).await {
                candidate = Some(full_doi);
                break;
            }
        }
        let full_doi = candidate.ok_or_else(|| {
            "Could not generate a unique DOI suffix after 3 attempts. Please try again.".to_string()
        })?;
        (Some(full_doi), None)
    } else {
        (None, Some(prefix))
    };

    let request_body = DataciteDoiRequest {
        data: DataciteDoiData {
            data_type: "dois",
            attributes: DataciteDoiAttributes {
                doi: doi_attr,
                prefix: prefix_attr,
                event: if publish {
                    Some("publish".to_string())
                } else {
                    None
                },
                metadata,
            },
        },
    };

    let url = format!("{}/dois", datacite_base_url(&env));
    let response = client
        .post(&url)
        .basic_auth(&repository_id, Some(&password))
        .header("Content-Type", "application/vnd.api+json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("DataCite API request failed: {e}"))?;

    let response = require_success(response).await?;

    let doi_response: DataciteDoiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse DataCite API response: {e}"))?;

    Ok(doi_response.data.attributes.doi)
}

// ── Commandes Tauri ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn map_hal_to_datacite(
    hal_metadata: HalMetadata,
) -> Result<DataciteMetadata, String> {
    map(hal_metadata)
}

// ── Helpers internes ────────────────────────────────────────────────────────

async fn fetch_datacite_credentials(
    store: &CredentialStore,
    env: String,
) -> Result<(String, String), String> {
    let store_arc = store.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let vault = store_arc.as_ref();
        let repository_id = get_vault_secret(vault, "datacite", &env, "repository_id")?;
        let password = get_vault_secret(vault, "datacite", &env, "password")?;
        Ok::<(String, String), String>((repository_id, password))
    })
    .await
    .map_err(|e| e.to_string())?
}

async fn require_success(response: reqwest::Response) -> Result<reqwest::Response, String> {
    if response.status().is_success() {
        return Ok(response);
    }
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    Err(format!("DataCite API returned status {status}: {body}"))
}

/// Vérifie si un DOI existe déjà dans DataCite.
/// Retourne `true` si le DOI est déjà pris, `false` s'il est disponible (404) ou en cas d'erreur réseau.
async fn doi_exists(
    client: &reqwest::Client,
    doi: &str,
    repository_id: &str,
    password: &str,
    env: &str,
) -> bool {
    let url = format!("{}/dois/{}", datacite_base_url(env), doi);
    match client
        .get(&url)
        .basic_auth(repository_id, Some(password))
        .send()
        .await
    {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

// ── Structures pour get_datacite_prefixes ──────────────────────────────────

#[derive(Deserialize, Debug)]
struct ClientPrefixRelationshipData {
    id: String,
}

#[derive(Deserialize, Debug)]
struct ClientPrefixRelationshipPrefix {
    data: ClientPrefixRelationshipData,
}

#[derive(Deserialize, Debug)]
struct ClientPrefixRelationships {
    prefix: ClientPrefixRelationshipPrefix,
}

#[derive(Deserialize, Debug)]
struct ClientPrefixItem {
    relationships: ClientPrefixRelationships,
}

#[derive(Deserialize, Debug)]
struct ClientPrefixesResponse {
    data: Vec<ClientPrefixItem>,
}

/// Récupère la liste des préfixes DOI disponibles pour un compte DataCite.
#[tauri::command]
pub async fn get_datacite_prefixes(
    store: State<'_, CredentialStore>,
    env: String,
) -> Result<Vec<String>, String> {
    let (repository_id, password) = fetch_datacite_credentials(&store, env.clone()).await?;

    let url = format!(
        "{}/client-prefixes?client-id={}&page[size]=100",
        datacite_base_url(&env),
        repository_id
    );
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .basic_auth(&repository_id, Some(&password))
        .header("Accept", "application/vnd.api+json")
        .send()
        .await
        .map_err(|e| format!("DataCite API request failed: {e}"))?;

    let response = require_success(response).await?;

    let parsed: ClientPrefixesResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse DataCite prefixes response: {e}"))?;

    let prefixes = parsed
        .data
        .into_iter()
        .map(|item| item.relationships.prefix.data.id)
        .collect();

    Ok(prefixes)
}

// ── Structures pour publish_doi ─────────────────────────────────────────────

#[derive(Serialize, Debug)]
struct PublishDoiAttributes {
    event: &'static str,
}

#[derive(Serialize, Debug)]
struct PublishDoiData {
    #[serde(rename = "type")]
    data_type: &'static str,
    attributes: PublishDoiAttributes,
}

#[derive(Serialize, Debug)]
struct PublishDoiRequest {
    data: PublishDoiData,
}

/// Publie un DOI en brouillon (passe son état à "findable").
#[tauri::command]
pub async fn publish_doi(
    store: State<'_, CredentialStore>,
    doi: String,
    env: String,
) -> Result<(), String> {
    let (repository_id, password) = fetch_datacite_credentials(&store, env.clone()).await?;

    let request_body = PublishDoiRequest {
        data: PublishDoiData {
            data_type: "dois",
            attributes: PublishDoiAttributes { event: "publish" },
        },
    };

    let url = format!("{}/dois/{}", datacite_base_url(&env), doi);
    let client = reqwest::Client::new();
    let response = client
        .put(&url)
        .basic_auth(&repository_id, Some(&password))
        .header("Content-Type", "application/vnd.api+json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("DataCite API request failed: {e}"))?;

    require_success(response).await?;

    Ok(())
}

/// Supprime un DOI en état brouillon (draft) sur DataCite.
#[tauri::command]
pub async fn delete_doi(
    store: State<'_, CredentialStore>,
    doi: String,
    env: String,
) -> Result<(), String> {
    let (repository_id, password) = fetch_datacite_credentials(&store, env.clone()).await?;

    let url = format!("{}/dois/{}", datacite_base_url(&env), doi);
    let client = reqwest::Client::new();
    let response = client
        .delete(&url)
        .basic_auth(&repository_id, Some(&password))
        .send()
        .await
        .map_err(|e| format!("DataCite API request failed: {e}"))?;

    require_success(response).await?;

    Ok(())
}

// ── Structures pour get_datacite_dois ──────────────────────────────────────

#[derive(Deserialize, Debug)]
struct DoiApiTitle {
    title: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DoiApiTypes {
    resource_type_general: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DoiApiIdentifier {
    identifier: String,
    identifier_type: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DoiApiAttributes {
    doi: String,
    titles: Vec<DoiApiTitle>,
    state: String,
    publication_year: Option<u32>,
    types: Option<DoiApiTypes>,
    identifiers: Option<Vec<DoiApiIdentifier>>,
}

#[derive(Deserialize, Debug)]
struct DoiApiEntry {
    attributes: DoiApiAttributes,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DoiApiMeta {
    total: u32,
    total_pages: u32,
    page: u32,
}

#[derive(Deserialize, Debug)]
struct DoiApiListResponse {
    data: Vec<DoiApiEntry>,
    meta: DoiApiMeta,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DoiListItem {
    pub doi: String,
    pub title: String,
    pub state: String,
    pub publication_year: Option<u32>,
    pub resource_type_general: Option<String>,
    pub hal_id: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DoiListMeta {
    pub total: u32,
    pub total_pages: u32,
    pub page: u32,
}

#[derive(Serialize, Debug)]
pub struct DoiListResponse {
    pub items: Vec<DoiListItem>,
    pub meta: DoiListMeta,
}

/// Récupère la liste des DOIs créés pour un compte DataCite.
#[tauri::command]
pub async fn get_datacite_dois(
    store: State<'_, CredentialStore>,
    env: String,
    page: u32,
    page_size: u32,
) -> Result<DoiListResponse, String> {
    let (repository_id, password) = fetch_datacite_credentials(&store, env.clone()).await?;

    let url = format!(
        "{}/dois?client-id={}&page[number]={}&page[size]={}",
        datacite_base_url(&env),
        repository_id,
        page,
        page_size
    );
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .basic_auth(&repository_id, Some(&password))
        .header("Accept", "application/vnd.api+json")
        .send()
        .await
        .map_err(|e| format!("DataCite API request failed: {e}"))?;

    let response = require_success(response).await?;

    let parsed: DoiApiListResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse DataCite DOI list response: {e}"))?;

    let items = parsed
        .data
        .into_iter()
        .map(|entry| {
            let title = entry
                .attributes
                .titles
                .into_iter()
                .next()
                .map(|t| t.title)
                .unwrap_or_default();
            let hal_id = entry
                .attributes
                .identifiers
                .and_then(|ids| {
                    ids.into_iter()
                        .find(|id| id.identifier_type.to_uppercase().contains("HAL"))
                        .map(|id| id.identifier)
                });
            DoiListItem {
                doi: entry.attributes.doi,
                title,
                state: entry.attributes.state,
                publication_year: entry.attributes.publication_year,
                resource_type_general: entry
                    .attributes
                    .types
                    .and_then(|t| t.resource_type_general),
                hal_id,
            }
        })
        .collect();

    Ok(DoiListResponse {
        items,
        meta: DoiListMeta {
            total: parsed.meta.total,
            total_pages: parsed.meta.total_pages,
            page: parsed.meta.page,
        },
    })
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hal_service::HalAuthor;
    use std::collections::HashMap;

    fn minimal_hal() -> HalMetadata {
        HalMetadata {
            hal_id: "hal-12345".to_string(),
            title: vec!["Test title".to_string()],
            authors: vec![HalAuthor {
                full_name: "Doe, John".to_string(),
                first_name: Some("John".to_string()),
                last_name: Some("Doe".to_string()),
                orcid: None,
                affiliations: vec![],
            }],
            publication_date: Some("2023-06-15".to_string()),
            doc_type: None,
            abstracts: vec![],
            keywords: vec![],
            languages: vec![],
            existing_doi: None,
            uri: Some("https://hal.science/hal-12345".to_string()),
            license: None,
            titles_by_lang: HashMap::new(),
            abstracts_by_lang: HashMap::new(),
            keywords_by_lang: HashMap::new(),
            anr_project_titles: vec![],
            anr_project_references: vec![],
        }
    }

    #[test]
    fn test_map_minimal_ok() {
        let result = map(minimal_hal());
        assert!(result.is_ok());
        let meta = result.unwrap();
        assert_eq!(meta.publication_year, 2023);
        assert_eq!(meta.url, "https://hal.science/hal-12345");
        assert_eq!(meta.publisher.name, "HAL");
    }

    #[test]
    fn test_subjects_none_when_keywords_empty() {
        let result = map(minimal_hal()).unwrap();
        assert!(result.subjects.is_none());
    }

    #[test]
    fn test_language_none_when_languages_empty() {
        let result = map(minimal_hal()).unwrap();
        assert!(result.language.is_none());
    }

    #[test]
    fn test_doc_type_mapping() {
        let cases = vec![
            ("ART", "JournalArticle"),
            ("COMM", "ConferencePaper"),
            ("THESE", "Dissertation"),
            ("OUV", "Book"),
        ];
        for (hal_type, expected) in cases {
            let mut hal = minimal_hal();
            hal.doc_type = Some(hal_type.to_string());
            let meta = map(hal).unwrap();
            let serialized = serde_json::to_string(&meta.types.resource_type_general).unwrap();
            assert!(
                serialized.contains(expected),
                "Expected {expected} for {hal_type}, got {serialized}"
            );
        }
    }

    #[test]
    fn test_doc_type_none_gives_other() {
        let mut hal = minimal_hal();
        hal.doc_type = None;
        let meta = map(hal).unwrap();
        let serialized = serde_json::to_string(&meta.types.resource_type_general).unwrap();
        assert!(serialized.contains("Other"));
    }

    /// Test d'intégration : récupère une fiche HAL réelle et affiche les métadonnées DataCite.
    ///
    /// Ignoré par défaut (appel réseau). Pour l'exécuter :
    ///   cargo test --lib hal_to_datacite -- --ignored --nocapture
    ///
    /// Par défaut teste hal-04043343. Pour changer l'identifiant :
    ///   HAL_ID=hal-XXXXXXX cargo test --lib hal_to_datacite -- --ignored --nocapture
    #[test]
    #[ignore]
    fn hal_to_datacite() {
        let hal_id = std::env::var("HAL_ID").unwrap_or_else(|_| "hal-05422868".to_string());
        let _prefix = std::env::var("DOI_PREFIX").unwrap_or_else(|_| "10.xxxx".to_string());

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");

        let hal_meta = rt
            .block_on(crate::hal_service::get_hal_metadata(
                hal_id.clone(),
                "prod".to_string(),
            ))
            .unwrap_or_else(|e| panic!("HAL fetch failed for {hal_id}: {e}"));

        println!("\n=== HalMetadata ===");
        println!("{}", serde_json::to_string_pretty(&hal_meta).unwrap());

        match map(hal_meta) {
            Ok(datacite) => {
                println!("\n=== DataciteMetadata ===");
                println!("{}", serde_json::to_string_pretty(&datacite).unwrap());
            }
            Err(e) => panic!("Mapping failed: {e}"),
        }
    }
}
