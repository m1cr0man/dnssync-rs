#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub domain: String,
    pub ignored_cidrs: String,
}
