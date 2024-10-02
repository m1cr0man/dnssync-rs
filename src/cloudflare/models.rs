use crate::common::{Manage, Match, Record, Update};

pub(super) const COMMENT_WATERMARK: &str = "Managed by DNSSync";
pub(super) const COMMENT_INSTANCE_PREFIX: &str = "instance:";

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

#[derive(serde::Deserialize)]
pub(super) struct DeleteResponse {}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub(super) struct DNSRecord {
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    pub content: String,
    pub comment: Option<String>,
    pub ttl: usize,
    // id is used to construct the URL, not part of the body.
    #[serde(skip_serializing)]
    pub id: String,
}

impl DNSRecord {
    pub(super) fn get_instance_id(&self) -> Option<&str> {
        self.comment.as_ref().and_then(|comment| {
            comment.find(COMMENT_INSTANCE_PREFIX).and_then(|pos| {
                let (_, instance_id) = comment.split_at(pos + COMMENT_INSTANCE_PREFIX.len());
                Some(instance_id.trim())
            })
        })
    }

    pub(super) fn set_instance_id(&mut self, instance_id: &str) {
        self.comment = Some(format!(
            "{COMMENT_WATERMARK} {COMMENT_INSTANCE_PREFIX}{instance_id}"
        ));
    }
}

impl Manage for DNSRecord {
    fn is_managed(&self) -> bool {
        self.comment
            .as_ref()
            .is_some_and(|c| c.contains(COMMENT_WATERMARK))
    }
}

impl Match for DNSRecord {
    fn matches(&self, other: &Self) -> bool {
        self.name == other.name && self.kind == other.kind
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
        self.kind.eq_ignore_ascii_case(&other.kind)
            && self.name.eq_ignore_ascii_case(&other.name)
            && self.content == other.content
    }
}

impl From<DNSRecord> for Record {
    fn from(value: DNSRecord) -> Self {
        Record {
            kind: value.kind.to_uppercase(),
            name: url::Host::Domain(value.name),
            content: value.content,
            source: super::FRONTEND_NAME.to_string(),
        }
    }
}

impl From<Record> for DNSRecord {
    fn from(value: Record) -> Self {
        Self {
            kind: value.kind.to_uppercase(),
            name: value.name.to_string(),
            content: value.content,
            comment: Some(COMMENT_WATERMARK.to_string()),
            ttl: 1,
            id: String::new(),
        }
    }
}
