use ravendb_client::{cluster_topology::ClusterTopologyInfo, DocumentStoreBuilder};
use tracing::subscriber::set_global_default;
// use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();
    let scheme: String = std::env::var("RAVEN_SCHEME").unwrap_or_else(|_| "http".to_string());

    let mut document_store = DocumentStoreBuilder::new();
    if scheme == "https" {
        document_store = document_store
            .set_client_certificate("ravendb-client_dev_cert.pem")
            .set_urls(&["https://a.free.damccull.ravendb.cloud"]);
    } else {
        document_store = document_store.set_urls(&["http://localhost:8080"]);
    }

    let document_store = document_store.build()?;

    let session = document_store.open_session().await?;
    match session.get_cluster_topology().await {
        Ok(topology_string) => {
            tracing::debug!("{}", &topology_string);
            let topo = serde_json::from_str::<ClusterTopologyInfo>(topology_string.as_str())?;
            println!("{:#?}", topo);
        }
        Err(e) => {
            tracing::error!("Error happened: {}", &e);
            return Err(e);
        }
    };

    Ok(())
}

fn setup_tracing() {
    // Redirect all `log`'s events to the subscriber
    LogTracer::init().expect("Failed to set logger");
    // Set up tracing
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    //let formatting_layer = BunyanFormattingLayer::new("ravendb-rs-demo".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        //.with(JsonStorageLayer)
        //.with(formatting_layer)
        .with(HierarchicalLayer::new(2));
    set_global_default(subscriber).expect("Failed to set subscriber");
}
