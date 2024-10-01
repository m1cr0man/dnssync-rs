#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub domain: String,
    pub excluded_cidrs: Option<String>,
    pub included_cidrs: Option<String>,
}
