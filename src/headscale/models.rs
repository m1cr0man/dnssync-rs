#[derive(serde::Deserialize)]
pub(super) struct MachinesResponse {
    pub machines: Vec<Machine>,
}

#[derive(serde::Deserialize)]
pub(super) struct Machine {
    pub id: String,
    pub name: String,
    pub user: UserData,
    #[serde(rename = "givenName")]
    pub given_name: String,
    #[serde(rename = "ipAddresses")]
    pub ip_addresses: Vec<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct UserData {
    pub id: String,
    pub name: String,
}
