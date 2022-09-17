use ravendb_client::DocumentStoreBuilder;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_tracing();

    let urls = ["https://a.free.damccull.ravendb.cloud"];
    let client = DocumentStoreBuilder::new()
        .set_client_certificate("free.damccull.client.certificate.pem")
        .require_https()
        .set_urls(&urls)
        .build()?;
    //dbg!(&client);
    let session = client.open_session().await?;
    let rdb_version = session.get_ravendb_version().await?;
    dbg!(&rdb_version);
    Ok(())
}

fn setup_tracing() {
    // Redirect all `log`'s events to the subscriber
    LogTracer::init().expect("Failed to set logger");
    // Set up tracing
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("ravendb-rs-demo".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");
}
