use std::{collections::HashMap, net::IpAddr};

use reqwest::Url;
use tokio::sync::oneshot;

use crate::{cluster_topology::ClusterTopologyInfo, raven_command::RavenCommand};

pub type DnsOverrides = HashMap<String, IpAddr>;

#[derive(Debug)]
pub enum DocumentStoreMessage {
    /// Executes the provided [`RavenCommand`].
    ExecuteRavenCommand {
        raven_command: RavenCommand,
        // TODO: Change this to a DocumentStoreError or maybe a RavenError
        respond_to: oneshot::Sender<Result<reqwest::Response, anyhow::Error>>,
    },
    GetServerAddress {
        respond_to: oneshot::Sender<Result<Url, anyhow::Error>>,
    },
    UpdateTopology,
}

#[derive(Clone, Copy, Debug)]
pub enum DocumentStoreState {
    /// [`DocumentStore`] was initialized but has since been closed.
    Closed,

    /// [`DocumentStore`] is initialized.
    Initialized,

    /// [`DocumentStore`] has not yet been initialized.
    Unitilialized,
}

/// Requests to initialize.
#[derive(Debug)]
pub struct DocumentStoreInitialConfiguration {
    //async_document_id_generator: Box<dyn AsyncDocumentIdGenerator>,
    pub(crate) client_identity: Option<reqwest::Identity>,
    pub(crate) cluster_topology: ClusterTopologyInfo,
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
