use std::collections::HashMap;

use rand::{seq::IteratorRandom, thread_rng};

use crate::{database_topology::DatabaseTopology, server_node::ServerNode};

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
    topology: Option<DatabaseTopology>,
    /// Counts the node failures
    node_failures: HashMap<ServerNode, u32>,
    /// Maintains a list of response times (in milliseconds) for each node in the topology
    node_response_speed_ms: HashMap<ServerNode, u32>,
}
impl NodeSelector {
    pub fn new(topology: Option<DatabaseTopology>) -> Self {
        Self {
            run_speed_test: false,
            topology,
            node_failures: HashMap::new(),
            node_response_speed_ms: HashMap::new(),
        }
    }

    /// Returns the fastest node available if one exists.
    pub fn get_fastest_node(&self) -> Option<ServerNode> {
        // TODO: actually return the fastest node
        // For now just return the first node
        self.get_preferred_node()
    }

    /// Returns a specific node for the given session id.
    pub fn get_node_by_session_id(&self, session_id: i32) -> Option<ServerNode> {
        // TODO: actually return the session_id node.
        // For now just return preferred node
        self.get_preferred_node()
    }

    /// Returns the currently preferred node.
    /// Right now this looks for the first node with 0 failures and returns it.
    /// On the off chance all nodes have failures, it returns
    pub fn get_preferred_node(&self) -> Option<ServerNode> {
        let x = self
            .node_failures
            .iter()
            .find(|(node, count)| **count == 0)
            .and_then(|(node_key, count)| {
                self.topology
                    .as_ref()
                    .map(|topology| topology.nodes[node_key].clone())
            });

        if x.is_some() {
            return x;
        }

        // If all nodes are marked with failures, just select one at random. This may still
        // be `None` if the topology is empty.
        // NOTE: JVM version rotates through an index but this lib uses a HashMap to store
        // the nodes and ordering is irrelevant, so a random choice makes more sense.
        self.select_random_node()
    }

    /// Returns the requested node by node tag.
    pub fn get_requested_node(&self, tag: String) -> Option<ServerNode> {
        // TODO: actually return the fastest node
        // For now just return the first node
        self.get_preferred_node()
    }

    /// Returns a random node if all are faulted.
    fn select_random_node(&self) -> Option<ServerNode> {
        if let Some(topology) = &self.topology {
            let mut rng = thread_rng();
            topology.nodes.values().choose(&mut rng).cloned()
        } else {
            None
        }
    }
}
