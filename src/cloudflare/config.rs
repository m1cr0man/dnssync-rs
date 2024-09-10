#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub api_key: String,
    pub domain: url::Host,
}
