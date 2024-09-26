use snafu::ResultExt;

use crate::common::BackendSnafu;

use super::BACKEND_NAME;

#[derive(serde::Deserialize)]
pub(super) struct Record {
    pub name: String,
    pub kind: String,
    pub content: String,
}

impl TryFrom<Record> for crate::common::Record {
    type Error = crate::common::Error;

    fn try_from(value: Record) -> crate::common::Result<Self> {
        let name = url::Host::parse(&value.name)
            .boxed_local()
            .context(BackendSnafu {
                backend: BACKEND_NAME,
                message: format!("Failed to parse record name {}", value.name),
            })?;
        Ok(Self {
            name,
            kind: value.kind.to_uppercase(),
            content: value.content,
        })
    }
}
