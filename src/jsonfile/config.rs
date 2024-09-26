use std::path::PathBuf;

#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub domain: String,
    pub source: PathBuf,
}
