//! The raven commands will be the only way to directly interact with the server
/// Although a command may exist that lets you artitrarily send a REST request
///
/// The user will create the command, either through a builder pattern or through a
/// set of structs that impl a trait and then pass it to a command executor
///
/// Common things all commands will need:
/// * Base url
/// * path to REST endpoint
/// * HTTP Method
/// * Body/payload
/// * headers
/// * if trait, a common 'execute' method
use reqwest::Method;
use url::Url;

#[derive(Debug)]
pub struct RavenCommand {
    pub base_server_url: Url,
    pub command: RavenCommandVariant,
}

impl RavenCommand {
    /// Returns a [`reqwest::Request`] for the specific [`RavenCommandVariant`].
    pub fn get_http_request(&self) -> anyhow::Result<reqwest::Request> {
        //TODO: Replace this client with one passed in somehow from the client cache
        let client = reqwest::Client::new();

        let request_config = RequestConfig {
            client,
            base_url: self.base_server_url.to_owned(),
        };

        // Handle specific command options
        let request = match &self.command {
            RavenCommandVariant::GetClusterTopology => {
                create_get_cluster_topology_request(request_config)?
            }
            RavenCommandVariant::GetAllDocumentsFromDatabase {
                database,
                page_size,
                start,
            } => create_get_all_documents_from_database_request(
                request_config,
                database.clone(),
                *page_size,
                *start,
            )?,
        };

        Ok(request)
    }
}

fn create_get_cluster_topology_request(config: RequestConfig) -> anyhow::Result<reqwest::Request> {
    let request = config
        .client
        .request(Method::GET, config.base_url.join("cluseter/topology")?)
        .build()?;
    Ok(request)
}

fn create_get_all_documents_from_database_request(
    config: RequestConfig,
    database: String,
    page_size: Option<i64>,
    start: Option<i64>,
) -> anyhow::Result<reqwest::Request> {
    //Create a vec to hold optional parts of the query string
    let mut query_string_parts = Vec::new();

    // Check if `page_size` and `start` are Some and add to the vec if so
    if let Some(page_size) = page_size {
        query_string_parts.push(format!("pageSize={}", page_size))
    }
    if let Some(start) = start {
        query_string_parts.push(format!("start={}", start))
    }

    // Make the full query string by joining the parts with &
    let query_string = query_string_parts.join("&");

    let mut url = config
        .base_url
        .join("databases/")?
        .join(format!("{}/", database).as_str())?
        .join("docs")?;
    url.set_query(Some(query_string.as_str()));

    let request = config.client.request(Method::GET, url).build()?;

    Ok(request)
}
/// Represents all operations that can be sent to the server.
/// Contained inside a [`RavenCommand`]. Holds all data relevant
/// to the specific command to be sent.
#[derive(Debug)]
pub enum RavenCommandVariant {
    GetClusterTopology,
    GetAllDocumentsFromDatabase {
        database: String,
        page_size: Option<i64>,
        start: Option<i64>,
    },
}

#[derive(Debug)]
pub struct RequestConfig {
    client: reqwest::Client,
    base_url: Url,
}
