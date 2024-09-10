use std::str::FromStr;

use crate::common::{
    Frontend, FrontendSnafu, Record, RequestSnafu, Result, RECORD_KIND_A, RECORD_KIND_AAAA,
};

use super::models::{Machine, MachinesResponse};
use snafu::ResultExt;

pub struct HeadscaleFrontend {
    domain: url::Host,
    api_key: String,
    machines_url: url::Url,
}

impl HeadscaleFrontend {
    fn convert_machine(&self, machine: &Machine) -> Result<Vec<Record>> {
        let mut records = Vec::with_capacity(machine.ip_addresses.len());
        for ip in machine.ip_addresses.iter() {
            let ip_addr = std::net::IpAddr::from_str(&ip)
                .boxed_local()
                .context(FrontendSnafu {
                    message: format!("Failed to parse ip {}", ip),
                })?;

            let kind = match ip_addr {
                std::net::IpAddr::V4(_) => RECORD_KIND_A,
                std::net::IpAddr::V6(_) => RECORD_KIND_AAAA,
            };

            records.push(Record {
                name: url::Host::Domain(format!("{}.{}", machine.given_name, self.domain)),
                kind: kind.to_string(),
                content: ip.clone(),
            });
        }

        Ok(records)
    }
}

impl Frontend for HeadscaleFrontend {
    fn read_records(&self) -> Result<Vec<Record>> {
        let response: MachinesResponse = ureq::get(self.machines_url.as_str())
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .call()
            .context(RequestSnafu {
                url: self.machines_url.as_str(),
                method: "GET",
            })?
            .into_json()
            .boxed_local()
            .context(FrontendSnafu {
                message: "Failed to deserialize response",
            })?;

        let mut records = Vec::new();
        for machine in response.machines {
            records.extend(self.convert_machine(&machine)?);
        }

        Ok(records)
    }
}

impl From<super::Config> for HeadscaleFrontend {
    fn from(mut value: super::Config) -> Self {
        value
            .base_url
            .path_segments_mut()
            .expect("base_url should be a HTTP URL")
            .extend(&["api", "v1", "machine"]);
        Self {
            domain: value.domain,
            api_key: value.api_key,
            machines_url: value.base_url,
        }
    }
}
