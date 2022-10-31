use std::collections::HashMap;

use crate::server_node::ServerNode;

#[derive(Debug, Default)]
pub struct Topology {
    pub etag: u64,
    pub nodes: HashMap<ServerNode, ServerNode>,
}
