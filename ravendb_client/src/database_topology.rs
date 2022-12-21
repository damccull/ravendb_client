use std::collections::{HashMap, HashSet};

use crate::server_node::ServerNode;

#[derive(Clone, Debug, Default)]
pub struct DatabaseTopology {
    /// Represents the latest version of the topology
    pub etag: u64,
    /// Holds the nodes.
    pub nodes: HashSet<ServerNode>,
    /// Counts the node failures
    pub node_failures: HashMap<ServerNode, u32>,
    /// Maintains a list of response times (in milliseconds) for each node in the topology
    pub node_response_speed_ms: HashMap<ServerNode, u32>,
}
