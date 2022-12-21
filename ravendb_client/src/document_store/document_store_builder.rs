use std::{collections::HashMap, fs::File, io::Read};

use reqwest::Url;
use tracing::instrument;

use crate::{DnsOverrides, DocumentStore, DocumentStoreError, DocumentStoreInitialConfiguration};

#[derive(Debug)]
pub struct DocumentStoreBuilder {
    client_certificate_path: Option<String>,
    database_name: Option<String>,
    dns_overrides: Option<DnsOverrides>,
    document_store_urls: Vec<String>,
    proxy_address: Option<String>,
}

impl DocumentStoreBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_dns_overrides(mut self, overrides: DnsOverrides) -> Self {
        tracing::trace!("Adding to dns_overrides: {:?}", &overrides);
        self.dns_overrides = Some(overrides);
        self
    }

    pub fn set_client_certificate(mut self, certificate_path: &str) -> Self {
        self.client_certificate_path = Some(certificate_path.to_string());
        self
    }

    pub fn set_proxy_address(mut self, proxy_address: &str) -> Self {
        self.proxy_address = Some(proxy_address.to_string());
        self
    }

    pub fn set_urls<T>(mut self, urls: &[T]) -> Self
    where
        T: AsRef<str>,
    {
        for u in urls {
            self.document_store_urls.push(u.as_ref().to_string());
        }
        self
    }

    pub fn set_database_name(mut self, database_name: &str) -> Self {
        self.database_name = Some(database_name.to_string());
        self
    }

    /// Initializes a new [`DocumentStoreActor`] and retuns a handle to it.
    ///
    /// Each call to this will create a new [`DocumentStoreActor`] and return a new handle to it.
    /// It is not recommended to create more that one per database cluster. This function is allowed
    /// to be called more than once to the builder can act as a template after being set up once.
    #[instrument(level = "debug", name = "Build DocumentStoreBuilder", skip(self))]
    pub fn build(&self) -> Result<DocumentStore, DocumentStoreError> {
        // Ensure DocumentStore URLs are valid and there is at least one
        if self.document_store_urls.is_empty() {
            tracing::error!(
                "No URLs were supplied and a document store can't exist without at least one"
            );
            return Err(DocumentStoreError::MissingUrlsError);
        }

        // Validate URLS
        let initial_urls = validate_urls(
            self.document_store_urls.as_slice(),
            self.client_certificate_path.is_some(),
        )?;

        // let topology_info = ClusterTopologyInfo {
        //     topology: ClusterTopology {
        //         all_nodes: initial_node_list,
        //         ..Default::default()
        //     },
        //     ..Default::default()
        // };

        let identity = match &self.client_certificate_path {
            Some(certpath) => {
                // Open and validate certificate, and create an identity from it
                let mut buf = Vec::new();
                File::open(certpath)
                    .map_err(|e| {
                        let err =
                            anyhow::anyhow!("Failed to open certificate file. Caused by: {}", e);
                        tracing::error!("{}", &err);
                        err
                    })?
                    .read_to_end(&mut buf)
                    .map_err(|e| {
                        let err =
                            anyhow::anyhow!("File was opened but unable to read. Caused by: {}", e);
                        tracing::error!("{}", err);
                        err
                    })?;
                let id = reqwest::Identity::from_pem(&buf).map_err(|e| {
                    let err = anyhow::anyhow!("Invalid pem file. Caused by: {}", e);
                    tracing::error!("{}", err);
                    err
                })?;
                Some(id)
            }
            None => None,
        };

        // Create an initial configuration for the DocumentStoreActor
        let initial_config = DocumentStoreInitialConfiguration {
            //async_document_id_generator: self.async_document_id_generator.clone(),
            client_identity: identity,
            // cluster_topology: topology_info,
            initial_urls: initial_urls.values().cloned().collect::<Vec<_>>(),
            database_name: self.database_name.clone(),
            dns_overrides: self.dns_overrides.clone(),
            proxy_address: self.proxy_address.clone(),
        };

        tracing::trace!("Initial Configuration: {:?}", &initial_config);

        Ok(DocumentStore::new(initial_config))
    }
}

#[allow(clippy::derivable_impls)] //TODO: Remove this allow when ready
impl Default for DocumentStoreBuilder {
    fn default() -> Self {
        // TODO: Create a default async id generator in the Default implementation

        Self {
            //async_document_id_generator: Box::new(AsyncMultiDatabaseHiLoIdGenerator::default()),
            client_certificate_path: None,
            database_name: None,
            dns_overrides: None,
            document_store_urls: Vec::new(),
            proxy_address: None,
        }
    }
}

/// Converts the provided URL strings to a [`Vec`] of [`Url`], ensuring they are a valid format.
///
/// Also ensures all provided URL strings use the same schema: either https or http, but never both within the
/// list.
#[instrument(level = "debug", name = "Validate URLs")]
fn validate_urls<T: std::fmt::Debug>(
    urls: &[T],
    require_https: bool,
) -> anyhow::Result<HashMap<String, Url>>
where
    T: AsRef<str>,
{
    //let mut clean_urls = Vec::new();

    //TODO: Check URLs are valid
    //TODO: Check all URLs are either http OR https, no mixing

    let clean_urls = urls
        .iter()
        .flat_map(|url| -> anyhow::Result<Url> { Ok(Url::parse(url.as_ref())?) })
        .map(|url| (url.to_string(), url))
        .collect::<HashMap<_, _>>();

    let desired_scheme = if require_https { "https" } else { "http" };

    for url in clean_urls.values().collect::<Vec<_>>() {
        if url.scheme() != desired_scheme {
            return Err(anyhow::anyhow!("Url does not have correct scheme: {}", url));
        }
    }

    Ok(clean_urls)
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use std::collections::HashMap;

    use url::Url;

    use crate::{DocumentStoreBuilder, DocumentStoreError};

    use super::validate_urls;

    #[test]
    fn validate_urls_returns_correct_HashMap_for_http_strings() {
        // Arrange
        let mut baseline_urls = HashMap::<String, Url>::new();
        baseline_urls.insert(
            "http://starwars.com/".to_string(),
            Url::parse("http://starwars.com").unwrap(),
        );
        baseline_urls.insert(
            "http://google.com/".to_string(),
            Url::parse("http://google.com").unwrap(),
        );

        let urls = vec!["http://starwars.com", "http://google.com"];

        // Act
        let result = validate_urls(urls.as_slice(), false).unwrap();
        // Assert
        assert_eq!(result, baseline_urls);
    }

    #[test]
    fn validate_urls_returns_correct_HashMap_for_https_strings() {
        // Arrange
        let mut baseline_urls = HashMap::<String, Url>::new();
        baseline_urls.insert(
            "https://starwars.com/".to_string(),
            Url::parse("https://starwars.com").unwrap(),
        );
        baseline_urls.insert(
            "https://google.com/".to_string(),
            Url::parse("https://google.com").unwrap(),
        );

        let urls = vec!["https://starwars.com", "https://google.com"];

        // Act
        let result = validate_urls(urls.as_slice(), true).unwrap();

        // Assert
        assert_eq!(result, baseline_urls);
    }

    #[test]
    fn validate_urls_fails_for_mixed_http_and_https_strings() {
        // Arrange
        let urls = vec!["https://starwars.com", "http://google.com"];

        // Assert
        assert!(validate_urls(urls.as_slice(), true).is_err());
        assert!(validate_urls(urls.as_slice(), false).is_err());
    }

    #[tokio::test]
    async fn documentstorebuilder_build_succeeds_for_valid_configuration() {
        // Arrange
        let urls = ["https://localhost:8080"];

        let document_store = DocumentStoreBuilder::new()
            .set_client_certificate("../ravendb-client_dev_cert.pem")
            .set_urls(&urls)
            .build();

        // Assert
        assert!(document_store.is_ok());
    }

    #[tokio::test]
    async fn documentstorebuilder_build_fails_for_invalid_pem() {
        // Arrange
        let urls = ["https://localhost:8080"];

        let document_store = DocumentStoreBuilder::new()
            // README.md is not a valid PEM file
            .set_client_certificate("../README.md")
            .set_urls(&urls)
            .build();

        // Assert
        assert!(document_store.is_err());
    }

    #[tokio::test]
    async fn documentstorebuilder_build_fails_if_no_urls() {
        let document_store = DocumentStoreBuilder::new()
            .set_client_certificate("../ravendb-client_dev_cert.pem")
            .build();

        assert!(
            document_store.is_err()
                && matches!(document_store, Err(DocumentStoreError::MissingUrlsError))
        );
    }
}
