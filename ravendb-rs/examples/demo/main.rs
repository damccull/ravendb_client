use ravendb_client::DocumentStoreBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let urls = ["https://a.free.damccull.ravendb.cloud"];
    let client = DocumentStoreBuilder::new()
        .set_client_certificate("free.damccull.client.certificate.pem")
        .require_https()
        .set_urls(&urls)
        .build()?;
    dbg!(&client);
    let session = client.open_session().await?;
    dbg!(&session);
    Ok(())
}
