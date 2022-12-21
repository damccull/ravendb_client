mod document_store_actor;
mod document_store_builder;
mod document_store_error;
mod document_store_handle;

pub use document_store_actor::*;
pub use document_store_builder::*;
pub use document_store_error::*;
pub use document_store_handle::*;

use reqwest::Url;
use std::{collections::HashMap, net::IpAddr};
use tokio::sync::oneshot;

use crate::request_executor::RequestExecutor;

pub type DnsOverrides = HashMap<String, IpAddr>;

#[derive(Debug)]
pub enum DocumentStoreMessage {
    /// Executes the provided [`RavenCommand`].
    // ExecuteRavenCommand {
    //     raven_command: RavenCommand,
    //     // TODO: Change this to a DocumentStoreError or maybe a RavenError
    //     respond_to: oneshot::Sender<Result<reqwest::Response, anyhow::Error>>,
    // },
    GetDatabase {
        respond_to: oneshot::Sender<Option<String>>,
    },
    GetRequestExecutor {
        database_name: Option<String>,
        respond_to: oneshot::Sender<Result<RequestExecutor, DocumentStoreError>>,
    },
    GetServerAddress {
        respond_to: oneshot::Sender<Result<Url, anyhow::Error>>,
    },
    // UpdateTopology,
}

/// Requests to initialize.
#[derive(Debug)]
pub struct DocumentStoreInitialConfiguration {
    //async_document_id_generator: Box<dyn AsyncDocumentIdGenerator>,
    pub(crate) client_identity: Option<reqwest::Identity>,
    // pub(crate) cluster_topology: ClusterTopologyInfo,
    pub(crate) initial_urls: Vec<Url>,
    pub(crate) database_name: Option<String>,
    pub(crate) dns_overrides: Option<DnsOverrides>,
    pub(crate) proxy_address: Option<String>,
}

// Placeholders below
#[derive(Debug)]
pub struct Conventions;
pub struct CertificatePlaceholder;

pub struct DatabaseChanges;
pub struct DatabaseChangesBuilder;

#[derive(Debug)]
pub struct DocumentSubscription;
