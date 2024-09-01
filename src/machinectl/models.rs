pub(super) type Machines = Vec<Machine>;

#[derive(serde::Deserialize)]
pub(super) struct Machine {
    #[serde(rename = "machine")]
    pub name: String,
    pub addresses: String,
}
