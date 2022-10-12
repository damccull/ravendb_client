use reqwest::Url;

use crate::cluster_topology::ClusterTopology;

#[derive(Debug, Clone)]
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

    pub fn set_cluster_tag(&mut self, tag: String) {
        self.cluster_tag = tag;
    }

    pub fn get_cluster_tag(&self) -> &str {
        self.cluster_tag.as_str()
    }

    pub fn set_server_role(&mut self, role: ServerRole) {
        self.server_role = role;
    }

    pub fn get_server_role(&self) -> ServerRole {
        self.server_role
    }

    pub fn supports_atomic_cluster_writes() {
        // Needs to check version number is >= 5.2
        todo!()
    }
}

pub fn create_server_nodes_from_topology(topology: ClusterTopology) -> Vec<ServerNode> {
    todo!()
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ServerRole {
    #[default]
    None,
    Promotable,
    Member,
    Rehab,
}
