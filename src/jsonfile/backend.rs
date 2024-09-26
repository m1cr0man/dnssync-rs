use std::{fs::File, io::BufReader, path::PathBuf};

use snafu::ResultExt;

use crate::common::{Backend, BackendSnafu, Record, Result};

const BACKEND_NAME: &str = "JSONFile";

pub struct JSONFileBackend {
    domain: String,
    source: PathBuf,
}

impl Backend for JSONFileBackend {
    fn get_domain(&self) -> String {
        return self.domain.to_owned();
    }

    fn read_records(&self) -> Result<Vec<Record>> {
        let file = File::open(&self.source)
            .boxed_local()
            .context(BackendSnafu {
                backend: BACKEND_NAME,
                message: format!("Failed to open source {}", self.source.display()),
            })?;

        let reader = BufReader::new(file);
        let mut records: Vec<Record> =
            serde_json::from_reader(reader)
                .boxed_local()
                .context(BackendSnafu {
                    backend: BACKEND_NAME,
                    message: format!("Failed to read records from source"),
                })?;

        // Ensure all the records are correctly cased
        records.iter_mut().for_each(|record| {
            record.kind.make_ascii_uppercase();
        });

        tracing::info!(
            backend = BACKEND_NAME,
            records = records.len(),
            "Read completed",
        );
        Ok(records.clone())
    }
}

impl From<super::Config> for JSONFileBackend {
    fn from(value: super::Config) -> Self {
        Self {
            domain: value.domain,
            source: value.source,
        }
    }
}
