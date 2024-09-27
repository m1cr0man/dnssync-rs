use crate::common::{Manage, Match, Record, Update};

pub(super) const COMMENT_WATERMARK: &str = "Managed by DNSSync.";
pub(super) const COMMENT_KEY_DOMAIN: &str = "sync_domain:";

fn domain_identifier(domain: &str) -> String {
    format!("{COMMENT_KEY_DOMAIN}{domain}")
}

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
    pub(super) fn in_domain(&self, domain: &str) -> bool {
        self.comment
            .as_ref()
            .is_some_and(|c| c.contains(&domain_identifier(domain)))
    }

    pub(super) fn set_domain(&mut self, domain: &str) {
        let val = domain_identifier(domain);
        match self.comment.as_mut() {
            Some(comment) => {
                // Remove any existing domain
                if let Some(pos) = comment.find(COMMENT_KEY_DOMAIN) {
                    let _ = comment.split_off(pos);
                }
                comment.push_str(&val);
            }
            None => self.comment = Some(format!("{COMMENT_WATERMARK} {}", val)),
        };
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
