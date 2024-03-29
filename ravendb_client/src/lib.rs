/*!
ravendb_client is a client library for the RavenDB document database.
It aims to compete with the existing libraries officially offered, but
using more rusty ways of doing things.

This library requires tokio and async, and uses the actor pattern to maintain
a single instance of the [`DocumentStore`] per cluster, as recommended by the
official libraries. This is designed to keep resource usage in your app to a minimum.

A [`DocumentSession`] can be requested from the [`DocumentStore`] to interact with the
database. It is considered a unit of work and all changes to a [`DocumentSession`] will
succeed or fail together.

# Example
// ```rust
// use ravendb_client::DocumentStore;

// let document_store = DocumentStore::new();
// let session = document_store.open_session();

// //...update entities here...

// session.save_changes();
// ```

When `session` is dropped, it'll close any open handles appropriately on its own.
*/

// TODO: REMOVE THIS
#![allow(dead_code, unreachable_code, unused_variables)]

mod document_conventions;
mod document_session;
mod document_store;

pub mod cluster_topology;
pub mod database_topology;
pub mod node_selector;
pub mod raven_command;
pub mod raven_command_generic;
pub mod ravendb_error;
mod request_executor;
mod server_node;

use std::{collections::HashMap, net::IpAddr};

pub use document_session::*;
pub use document_store::*;

pub type DnsOverrides = HashMap<String, IpAddr>;

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {}
