use reqwest::Url;

#[derive(Debug)]
pub struct ServerNode {
    url: Url,
    database: String,
    cluster_tag: String,
    server_role: ServerRole,
}
impl ServerNode {
    pub fn new(url: Url, database: String) -> Self {
        Self {
            url,
            database,
            cluster_tag: String::default(),
            server_role: ServerRole::default(),
        }
    }
}

#[derive(Debug, Default)]
pub enum ServerRole {
    #[default]
    None,
    Promotable,
    Member,
    Rehab,
}
