pub trait RavenCommand {
    type Result;
    fn get_request(client: reqwest::Client) -> reqwest::Request;
}

pub struct ExampleRavenCommand;

impl RavenCommand for ExampleRavenCommand {
    type Result = ExampleRavenCommandResult;

    fn get_request(client: reqwest::Client) -> reqwest::Request {
        todo!()
    }
}

pub struct ExampleRavenCommandResult {
    data: bool,
}
