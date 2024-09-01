#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub smtp_host: String,

    pub smtp_username: String,

    pub smtp_password: String,

    pub send_retries: usize,

    pub fixed_subject: Option<String>,
}
