use std::collections::HashMap;

use reqwest::Url;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ClusterTopologyInfo {
    #[serde(rename = "@metadata")]
    pub metadata: HashMap<String, String>, // TODO: Determine if needed or useful and delete if not
    pub topology: ClusterTopology,
    pub etag: i64,
    pub leader: String,
    pub leader_ship_duration: Option<i64>, // TODO: Determine if needed or useful and delete if not
    pub current_state: String,             // TODO: Determine if needed or useful and delete if not
    pub node_tag: String,
    pub current_term: i64, // TODO: Determine if needed or useful and delete if not
    pub node_license_details: HashMap<String, NodeLicenseDetails>, // TODO: Determine if needed or useful and delete if not
    pub last_state_change_reason: String, // TODO: Determine if needed or useful and delete if not
    pub status: HashMap<String, NodeStatus>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct ClusterTopology {
    pub topology_id: String,
    pub all_nodes: HashMap<String, Url>,
    pub members: HashMap<String, Url>,
    pub promotables: HashMap<String, Url>,
    pub watchers: HashMap<String, Url>,
    pub last_node_id: Option<String>,
    pub etag: i64,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct NodeLicenseDetails {
    pub utilized_cores: Option<i64>,
    pub max_utilized_cores: Option<i64>,
    pub number_of_cores: Option<i32>,
    pub installed_memory_in_gb: Option<f64>,
    pub usable_memory_in_gb: Option<f64>,
    pub build_info: BuildInfo,
    pub os_info: OsInfo,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct BuildInfo {
    pub product_version: Option<String>,
    pub build_version: Option<i64>,
    pub commit_hash: Option<String>,
    pub full_version: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct OsInfo {
    pub r#type: Option<String>,
    pub full_name: Option<String>,
    pub version: Option<String>,
    pub build_version: Option<String>,
    pub is_64_bit: bool,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct NodeStatus {
    pub name: Option<String>,
    pub connected: bool,
    pub error_details: Option<String>,
    pub last_sent: Option<String>,  //TODO: Make this a DateTime
    pub last_reply: Option<String>, //TODO: Make this a DateTime
    pub last_sent_message: Option<String>,
    pub last_matching_index: Option<i64>,
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
