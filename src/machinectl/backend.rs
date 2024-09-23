use std::str::FromStr;

use super::models::{Machine, Machines};
use crate::common::{
    Backend, BackendSnafu, ConfigSnafu, Record, Result, RECORD_KIND_A, RECORD_KIND_AAAA,
};
use snafu::ResultExt;

const BACKEND_NAME: &str = "Machinectl";

pub struct MachinectlBackend {
    domain: String,
    ignored_cidrs: Vec<cidr::IpCidr>,
}

impl MachinectlBackend {
    fn convert_machine(&self, machine: &Machine) -> Result<Vec<Record>> {
        let mut records = Vec::new();
        for ip in machine.addresses.split("\n") {
            let ip_addr = std::net::IpAddr::from_str(ip)
                .boxed_local()
                .context(BackendSnafu {
                    backend: BACKEND_NAME,
                    message: format!("Failed to parse ip {}", ip),
                })?;

            if self
                .ignored_cidrs
                .iter()
                .any(|ignored_cidr| ignored_cidr.contains(&ip_addr))
            {
                continue;
            }

            let kind = match ip_addr {
                std::net::IpAddr::V4(_) => RECORD_KIND_A,
                std::net::IpAddr::V6(_) => RECORD_KIND_AAAA,
            };

            records.push(Record {
                name: url::Host::Domain(format!("{}.{}", machine.name, self.domain)),
                kind: kind.to_string(),
                content: ip.to_string(),
            });
        }

        Ok(records)
    }
}

impl Backend for MachinectlBackend {
    fn read_records(&self) -> Result<Vec<Record>> {
        let output = std::process::Command::new("machinectl")
            .args(["list", "-o", "json"])
            .output()
            .boxed_local()
            .context(BackendSnafu {
                backend: BACKEND_NAME,
                message: "Failed to run machinectl list",
            })?;

        let data: Machines =
            serde_json::from_str(&String::from_utf8(output.stdout).boxed_local().context(
                BackendSnafu {
                    backend: BACKEND_NAME,
                    message: "Failed to decode stdout",
                },
            )?)
            .boxed_local()
            .context(BackendSnafu {
                backend: BACKEND_NAME,
                message: "Failed to parse machinectl list output",
            })?;

        let mut records = Vec::new();
        for machine in data {
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

impl From<super::Config> for MachinectlBackend {
    fn from(value: super::Config) -> Self {
        // Unfortunately config-rs makes it difficult to mix
        // strings and vec of strings, so we have to parse ourselves
        Self {
            domain: value.domain,
            ignored_cidrs: value
                .ignored_cidrs
                .split(",")
                .map(|v| {
                    cidr::IpCidr::from_str(v.trim())
                        .map_err(|_| {
                            ConfigSnafu {
                                message: format!("Invalid CIDR {v}"),
                                prefix: "machinectl.ignored_cidrs",
                            }
                            .build()
                        })
                        .unwrap()
                })
                .collect(),
        }
    }
}
