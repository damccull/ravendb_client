//!

use std::fmt::Display;

use url::Url;

/// The raven commands will be the only way to directly interact with the server
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

#[derive(Debug)]
pub struct RavenCommand {
    pub base_server_url: Url,
    pub http_method: HttpMethod,
    pub headers: Vec<Header>,
    pub command: RavenCommandOption,
}

impl RavenCommand {
    pub fn get_http_request(&self) -> anyhow::Result<reqwest::Request> {
        // Create the base request.
        let base_request =
            reqwest::Client::new().request(self.http_method.into(), "http://placeholder");
        // Handle specific command options
        let request = match self.command {
            RavenCommandOption::GetClusterTopology => {
                let mut request = base_request.build()?;
                *request.url_mut() = self.base_server_url.join("cluster/topology")?;
                request
            }
        };

        Ok(request)
    }
}

#[derive(Debug)]
pub enum RavenCommandOption {
    GetClusterTopology,
}

#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl From<HttpMethod> for reqwest::Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Delete => reqwest::Method::DELETE,
        }
    }
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let outstring = match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
        };
        f.write_str(outstring)
    }
}

#[derive(Debug, Clone)]
pub struct Header(String, String);
