use std::{thread, time::Duration};

use ravendb_client::DocumentStoreBuilder;
use tracing::{instrument, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
// use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();
    let span = tracing::info_span!("Starting");
    span.in_scope(|| async {
        tracing::info!("Starting application");
        run().await
    })
    .await?;

    thread::sleep(Duration::from_secs(2));
    Ok(())
}

#[instrument(level = "info", name = "Running")]
async fn run() -> anyhow::Result<()> {
    let scheme: String = std::env::var("RAVEN_SCHEME").unwrap_or_else(|_| "http".to_string());

    let mut document_store = DocumentStoreBuilder::new();
    if scheme == "https" {
        tracing::info!("`RAVEN_SCHEME` set to 'https'. Using pem file.");
        document_store = document_store
            .set_client_certificate("ravendb-client_dev_cert.pem")
            .set_urls(&["https://a.free.damccull.ravendb.cloud"]);
    } else {
        tracing::warn!("`RAVEN_SCHEME` not set or set to 'http'. Connecting insecurly and without authentication.");
        document_store = document_store.set_urls(&["http://localhost:8080"]);
    }

    let document_store = document_store.build()?;
    tracing::info!("DocumentStore created.");

    let session = document_store.open_session().await?;
    // match session.get_cluster_topology().await {
    //     Ok(topology) => {
    //         tracing::trace!("{:?}", &topology);
    //     }
    //     Err(e) => {
    //         tracing::error!("Error happened: {}", &e);
    //         return Err(e);
    //     }
    // };

    match session
        .get_all_documents_for_database("sample", Some(1), None)
        .await
    {
        Ok(topology) => {
            tracing::trace!("{:?}", &topology);
        }
        Err(e) => {
            tracing::error!("Error happened: {}", &e);
            return Err(e);
        }
    };

    thread::sleep(Duration::from_secs(2));

    match session
        .get_all_documents_for_database("sample", Some(1), None)
        .await
    {
        Ok(topology) => {
            tracing::trace!("{:?}", &topology);
        }
        Err(e) => {
            tracing::error!("Error happened: {}", &e);
            return Err(e);
        }
    };

    Ok(())
}

fn setup_tracing() {
    // Set up tracing
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("ravendb-rs-demo".into(), std::io::sink);
    //let heirarchical_layer = HierarchicalLayer::new(2);

    let tracing_formatter = tracing_subscriber::fmt::layer().pretty();
    let subscriber = Registry::default()
        .with(env_filter)
        .with(tracing_formatter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    //.with(heirarchical_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");

    // Redirect all `log`'s events to the subscriber
    LogTracer::init().expect("Failed to set logger");
}
