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
    update_fastest_node_timer: bool,
    state: NodeSelectorState,
}
impl NodeSelector {
    pub fn new(topology: Option<Topology>) -> Self {
        Self {
            update_fastest_node_timer: false,
            state: NodeSelectorState {
                topology: todo!(),
                nodes: todo!(),
            },
        }
    }
}

#[derive(Debug)]
pub struct NodeSelectorState {
    // Note: JVM version uses atomics here to thread-safely handle counters.
    // The actor-based architecture of the RequestExecutor means this isn't necessary
    // in rust since this will only ever be handled by a single thread.
    topology: Topology,
    nodes: Vec<ServerNode>,
}
