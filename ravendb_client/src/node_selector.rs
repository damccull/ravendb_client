use crate::{server_node::ServerNode, topology::Topology};

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
