#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub domain: String,
    pub api_key: String,
    pub base_url: url::Url,
    pub add_user_suffix: bool,
}
