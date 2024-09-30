use std::{fs::File, io::BufReader, path::PathBuf};

use snafu::ResultExt;

use crate::common::{self, BackendSnafu, Record, Result};

pub const BACKEND_NAME: &str = "JSONFile";

pub struct Backend {
    source: PathBuf,
}

impl common::Backend for Backend {
    fn read_records(&self) -> Result<Vec<Record>> {
        let file = File::open(&self.source)
            .boxed_local()
            .context(BackendSnafu {
                backend: BACKEND_NAME,
                message: format!("Failed to open source {}", self.source.display()),
            })?;

        let reader = BufReader::new(file);
        let records: Vec<super::models::Record> = serde_json::from_reader(reader)
            .boxed_local()
            .context(BackendSnafu {
                backend: BACKEND_NAME,
                message: format!("Failed to read records from source"),
            })?;

        // Convert between jsonfile::Record and common::Record
        let records = records
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<Record>>>()?;

        tracing::info!(
            backend = BACKEND_NAME,
            records = records.len(),
            "Read completed",
        );
        Ok(records.clone())
    }
}

impl From<super::Config> for Backend {
    fn from(value: super::Config) -> Self {
        Self {
            source: value.source,
        }
    }
}
