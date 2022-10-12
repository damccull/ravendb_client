use ravendb_client::{
    raven_command::{RavenCommand, RavenCommandVariant},
    ravendb_error::RavenDbError,
};
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let raven_command = RavenCommand {
        base_server_url: Url::parse("https://free.a.damccull.ravendb.cloud")?,
        command: RavenCommandVariant::GetClusterTopology,
    };
    let request = raven_command.get_http_request()?;

    let e: Result<(), RavenDbError> = Err(RavenDbError::DatabaseDoesNotExist("MyDb".to_string()));
    dbg!(request, e);
    Ok(())
}
