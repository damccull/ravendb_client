use std::collections::HashMap;

use crate::{server_node::ServerNode, topology::Topology};

///! Requirements for the NodeSelector
/// 1. Maintain the following state:
///    a. Current topology
///    b. Number of failures per node
///    c. Speed of each node (used to determine fastest)
///    d. Fastest node
///    e. Default node to try if all nodes in the server have faults
/// 2. Provide a speed test capability for all nodes
/// 3. Return the fastest node
/// 4. Return a specific node
/// 5. Return a "preferred" node
/// 6. Return the topology if requested

#[derive(Debug)]
pub struct NodeSelector {
    /// Whether or not to run speed tests
    run_speed_test: bool,
    /// Holds the topology
    topology: Option<Topology>,
    /// Counts the node failures
    node_failures: HashMap<ServerNode, u32>,
    /// Maintains a list of response times (in milliseconds) for each node in the topology
    node_response_speed_ms: HashMap<ServerNode, u32>,
}
impl NodeSelector {
    pub fn new(topology: Option<Topology>) -> Self {
        Self {
            run_speed_test: false,
            topology: None,
            node_failures: HashMap::new(),
            node_response_speed_ms: HashMap::new(),
        }
    }

    /// Returns the fastest node available if one exists.
    pub fn getFastestNode() -> Option<ServerNode> {
        todo!()
    }

    /// Returns a specific node for the given session id.
    pub fn getNodeBySessionId(session_id: i32) -> Option<ServerNode> {
        todo!()
    }

    /// Returns the currently preferred node.
    pub fn getPreferredNode() -> Option<ServerNode> {
        todo!()
    }

    /// Returns the requested node by node tag.
    pub fn getRequestedNode(tag: String) -> Option<ServerNode> {
        todo!()
    }
}
