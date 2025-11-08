#[derive(serde::Deserialize)]
pub(super) struct NodesResponse {
    pub nodes: Vec<Node>,
}

#[derive(serde::Deserialize)]
pub(super) struct Node {
    pub user: UserData,
    #[serde(rename = "givenName")]
    pub given_name: String,
    #[serde(rename = "ipAddresses")]
    pub ip_addresses: Vec<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct UserData {
    pub name: String,
}
