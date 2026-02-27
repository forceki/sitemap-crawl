use reqwest::Client;

use crate::config::DEFAULT_TIMEOUT;

pub fn build_client() -> reqwest::Result<Client> {
    Client::builder()
        .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT))
        .build()
}
