#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub poll_interval: usize,
}
