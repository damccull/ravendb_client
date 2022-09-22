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
        // Create the base request. This specifies a placeholder url and a GET method
        // by default. In each command, replace the url by joining the endpoint path to
        // the server base url. Also replace the method with the right one for the command.
        let base_request = reqwest::Client::new().request(Method::GET, "http://placeholder");

        tracing::debug!("{:?}", base_request);
        // Handle specific command options
        let request = match &self.command {
            RavenCommandVariant::GetClusterTopology => {
                let mut request = base_request.build()?;
                *request.url_mut() = self.base_server_url.join("cluster/topology")?;
                request
            }
            RavenCommandVariant::GetAllDocumentsFromDatabase {
                database,
                page_size,
                start,
            } => {
                let mut request = base_request.build()?;
                *request.url_mut() = self
                    .base_server_url
                    .join("databases/")?
                    .join(format!("{}/", database).as_str())?
                    .join("docs")?;

                let mut query_string_parts = Vec::new();
                if let Some(page_size) = page_size {
                    query_string_parts.push(format!("pageSize={}", page_size))
                }
                if let Some(start) = start {
                    query_string_parts.push(format!("start={}", start))
                }
                let query_string = query_string_parts.join("&");

                request.url_mut().set_query(Some(&query_string));

                request
            }
        };

        Ok(request)
    }
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
