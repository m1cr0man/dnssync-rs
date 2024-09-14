#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub domain: url::Host,
    pub api_key: String,
    pub base_url: url::Url,
    pub add_user_prefix: bool,
}
