use reqwest::Client;

use crate::config::{DEFAULT_TIMEOUT, DEFAULT_USER_AGENT};

pub fn build_client() -> reqwest::Result<Client> {
    Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT))
        .build()
}
