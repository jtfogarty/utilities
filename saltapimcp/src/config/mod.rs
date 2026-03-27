use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Salt API base URL (default: http://127.0.0.1:8000)
    #[arg(long, env = "SALT_API_URL", default_value = "http://127.0.0.1:8000")]
    pub salt_api_url: String,

    /// Salt username
    #[arg(long, env = "SALT_USER")]
    pub salt_user: String,

    /// Salt password
    #[arg(long, env = "SALT_PASS")]
    pub salt_pass: String,

    /// Eauth method (pam, ldap, file, etc.)
    #[arg(long, env = "SALT_EAUTH", default_value = "pam")]
    pub salt_eauth: String,
}