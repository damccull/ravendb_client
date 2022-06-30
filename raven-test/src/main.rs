use ravendb_client::DocumentStoreBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let urls = ["https://a.free.damccull.ravendb.cloud"];
    let _client = DocumentStoreBuilder::new()
        .set_client_certificate("free.damccull.client.certificate.pem")
        .require_https()
        .set_urls(&urls)
        .build()?;
    dbg!(_client);
    Ok(())
}
