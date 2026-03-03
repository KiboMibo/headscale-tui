use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_url: String,
    pub api_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let server_url = env::var("HEADSCALE_URL")
            .or_else(|_| env::var("HEADSCALE_SERVER"))
            .map_err(|_| "HEADSCALE_URL environment variable not set".to_string())?;

        let api_key = env::var("HEADSCALE_API_KEY")
            .map_err(|_| "HEADSCALE_API_KEY environment variable not set".to_string())?;

        let server_url = server_url.trim_end_matches('/').to_string();

        Ok(Config {
            server_url,
            api_key,
        })
    }

    pub fn from_args(server_url: String, api_key: String) -> Self {
        let server_url = server_url.trim_end_matches('/').to_string();
        Config {
            server_url,
            api_key,
        }
    }
}
