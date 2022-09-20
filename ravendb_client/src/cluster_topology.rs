use std::collections::HashMap;

use reqwest::Url;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
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
    pub node_license_details: HashMap<String, NodeLicenseDetails>,
    pub last_state_change_reason: String,
    pub status: HashMap<String, NodeStatus>,
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
#[serde(rename_all = "PascalCase")]
pub struct NodeLicenseDetails {
    pub utilized_cores: i64,
    pub max_utilized_cores: Option<i64>,
    pub number_of_cores: i32,
    pub installed_memory_in_gb: f64,
    pub usable_memory_in_gb: f64,
    pub build_info: BuildInfo,
    pub os_info: OsInfo,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct BuildInfo {
    pub product_version: String,
    pub build_version: i64,
    pub commit_hash: String,
    pub full_version: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct OsInfo {
    pub r#type: String,
    pub full_name: String,
    pub version: String,
    pub build_version: String,
    pub is_64_bit: bool,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct NodeStatus {
    pub name: Option<String>,
    pub connected: bool,
    pub error_details: Option<String>,
    pub last_sent: String,
    pub last_sent_message: String,
    pub last_matching_index: i64,
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
