use reqwest::Url;

use crate::cluster_topology::ClusterTopology;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ServerNode {
    pub url: Url,
    pub database: String,
    pub cluster_tag: String,
    pub server_role: ServerRole,
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

    pub fn supports_atomic_cluster_writes() {
        // Needs to check version number is >= 5.2
        todo!()
    }
}

pub fn create_server_nodes_from_cluster_topology(topology: ClusterTopology) -> Vec<ServerNode> {
    todo!()
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ServerRole {
    #[default]
    None,
    Promotable,
    Member,
    Rehab,
}
