use oauth2::basic::BasicClient;
use reqwest::Client;

#[derive(Clone)]
pub struct AppState {
    pub oauth: BasicClient,
    pub http_client: Client,
}

impl AppState {
    pub fn new(oauth: BasicClient) -> Self {
        Self { oauth, http_client: Client::new() }
    }
}
