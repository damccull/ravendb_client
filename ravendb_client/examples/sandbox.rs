use ravendb_client::{
    raven_command::{RavenCommand, RavenCommandVariant},
    ravendb_error::RavenDbError,
};
use reqwest::{Client, Response};
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let raven_command = RavenCommand {
        base_server_url: Url::parse("https://a.free.damccull.ravendb.cloud/")?,
        command: RavenCommandVariant::GetClusterTopology,
    };
    let request = raven_command.get_http_request()?;

    let client = Client::new();
    let f = client.execute(request);
    let r:Result<Response,reqwest::Error> = tokio::spawn(async move {
        f.await
    }).await.unwrap();
    dbg!(r);

    let e: Result<(), RavenDbError> = Err(RavenDbError::DatabaseDoesNotExist("MyDb".to_string()));
    //dbg!(request, e);
    Ok(())
}
