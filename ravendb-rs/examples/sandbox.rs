use ravendb_client::raven_command::{HttpMethod, RavenCommand, RavenCommandOption};
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let raven_command = RavenCommand {
        base_server_url: Url::parse("https://free.a.damccull.ravendb.cloud")?,
        http_method: HttpMethod::Get,
        headers: Vec::new(),
        command: RavenCommandOption::GetClusterTopology,
    };
    let request = raven_command.get_http_request()?;
    dbg!(request);
    Ok(())
}
