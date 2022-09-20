use std::collections::HashMap;

use reqwest::Url;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct ClusterTopologyInfo {
    #[serde(rename = "@metadata")]
    pub metadata: HashMap<String, String>,
    pub topology: Topology,
    pub etag: i64,
    pub leader: String,
    pub leader_ship_duration: i64,
    pub current_state: String,
    pub node_tag: String,
    pub current_term: i64,
    pub node_license_details: HashMap<String, String>,
    pub last_state_change_reason: String,
    pub status: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Topology {
    pub topology_id: String,
    pub all_nodes: HashMap<String, Url>,
    pub members: HashMap<String, Url>,
    pub promotables: HashMap<String, Url>,
    pub watchers: HashMap<String, Url>,
    pub last_node_id: String,
    pub etag: i64,
}

#[derive(Debug, Deserialize, Default)]
pub enum NodeState {
    #[default]
    Undefined,
    Passive,
    Candidate,
    Follower,
    #[serde(rename = "Leader-Elect")]
    LeaderElect,
    Leader,
}
