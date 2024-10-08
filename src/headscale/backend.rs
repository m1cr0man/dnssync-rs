use std::str::FromStr;

use crate::common::{
    self, key_file_or_string, BackendSnafu, Record, RequestSnafu, Result, RECORD_KIND_A,
    RECORD_KIND_AAAA,
};

use super::models::{Machine, MachinesResponse};
use snafu::ResultExt;

pub const BACKEND_NAME: &str = "Headscale";

pub struct Backend {
    domain: String,
    add_user_suffix: bool,
    api_key: String,
    machines_url: url::Url,
}

impl Backend {
    fn convert_machine(&self, machine: &Machine) -> Result<Vec<Record>> {
        let mut records = Vec::with_capacity(machine.ip_addresses.len());
        for ip in machine.ip_addresses.iter() {
            let ip_addr = std::net::IpAddr::from_str(&ip)
                .boxed_local()
                .context(BackendSnafu {
                    backend: BACKEND_NAME,
                    message: format!("Failed to parse ip {}", ip),
                })?;

            let kind = match ip_addr {
                std::net::IpAddr::V4(_) => RECORD_KIND_A,
                std::net::IpAddr::V6(_) => RECORD_KIND_AAAA,
            };

            let name = match self.add_user_suffix {
                true => url::Host::Domain(format!(
                    "{}.{}.{}",
                    machine.given_name, machine.user.name, self.domain
                )),
                false => url::Host::Domain(format!("{}.{}", machine.given_name, self.domain)),
            };

            records.push(Record {
                name,
                kind: kind.to_string(),
                content: ip.clone(),
                source: BACKEND_NAME.to_string(),
            });
        }

        Ok(records)
    }
}

impl common::Backend for Backend {
    fn read_records(&self) -> Result<Vec<Record>> {
        tracing::debug!(
            url = self.machines_url.as_str(),
            method = "GET",
            backend = "headscale",
            "Sending request"
        );
        let response: MachinesResponse = ureq::get(self.machines_url.as_str())
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .call()
            .context(RequestSnafu {
                url: self.machines_url.as_str(),
                method: "GET",
            })?
            .into_json()
            .boxed_local()
            .context(BackendSnafu {
                backend: BACKEND_NAME,
                message: "Failed to deserialize response",
            })?;

        let mut records = Vec::new();
        for machine in response.machines {
            records.extend(self.convert_machine(&machine)?);
        }

        tracing::info!(
            backend = BACKEND_NAME,
            records = records.len(),
            "Read completed",
        );

        Ok(records)
    }
}

impl From<super::Config> for Backend {
    fn from(mut value: super::Config) -> Self {
        value
            .base_url
            .path_segments_mut()
            .expect("base_url should be a HTTP URL")
            .extend(&["api", "v1", "machine"]);

        let api_key = key_file_or_string(value.api_key, BACKEND_NAME.into()).unwrap();

        Self {
            domain: value.domain,
            add_user_suffix: value.add_user_suffix,
            api_key,
            machines_url: value.base_url,
        }
    }
}
