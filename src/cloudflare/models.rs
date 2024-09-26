use crate::common::{Manage, Match, Record, Update};

pub(super) const DNS_RECORD_COMMENT: &str = "Managed by DNSSync";

#[derive(serde::Deserialize)]
pub(super) struct APIError {
    pub code: usize,
    pub message: String,
}

#[derive(serde::Deserialize)]
pub(super) struct PaginatedResponse<T> {
    pub success: bool,
    pub result: Vec<T>,
    pub errors: Vec<APIError>,
    pub result_info: Option<ResultInfo>,
}

#[derive(serde::Deserialize)]
pub(super) struct WriteResponse<T> {
    pub success: bool,
    pub result: T,
    pub errors: Vec<APIError>,
}

#[derive(serde::Deserialize)]
pub(super) struct ResultInfo {
    pub count: usize,
    pub page: usize,
    pub per_page: usize,
}

#[derive(serde::Deserialize)]
pub(super) struct Zone {
    pub name: String,
    pub id: String,
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub(super) struct DNSRecord {
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    pub content: String,
    pub comment: Option<String>,
    pub ttl: usize,
    pub id: String,
}

impl Manage for DNSRecord {
    fn is_managed(&self) -> bool {
        return self.comment == Some(DNS_RECORD_COMMENT.to_string());
    }
}

impl Match for DNSRecord {
    fn matches(&self, other: &Self) -> bool {
        return self.name == other.name && self.kind == other.kind;
    }
}

impl Update for DNSRecord {
    fn update(mut self, authority: Record) -> Self {
        self.content = authority.content;
        self
    }
}

impl PartialEq for DNSRecord {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.name == other.name && self.content == other.content
    }
}

impl From<DNSRecord> for Record {
    fn from(value: DNSRecord) -> Self {
        Record {
            kind: value.kind,
            name: url::Host::Domain(value.name),
            content: value.content,
        }
    }
}

impl From<Record> for DNSRecord {
    fn from(value: Record) -> Self {
        Self {
            kind: value.kind,
            name: value.name.to_string(),
            content: value.content,
            comment: Some(DNS_RECORD_COMMENT.to_string()),
            ttl: 10,
            id: String::new(),
        }
    }
}
