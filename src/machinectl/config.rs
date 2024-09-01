#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub domain: url::Host,
    pub ignored_cidrs: Option<Vec<cidr::IpCidr>>,
}
