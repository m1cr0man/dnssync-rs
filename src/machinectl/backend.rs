use std::str::FromStr;

use super::models::{Machine, Machines};
use crate::common::{
    self, BackendSnafu, ConfigSnafu, Record, Result, RECORD_KIND_A, RECORD_KIND_AAAA,
};
use snafu::ResultExt;

pub const BACKEND_NAME: &str = "Machinectl";

pub struct Machinectl {
    domain: String,
    excluded_cidrs: Vec<cidr::IpCidr>,
    included_cidrs: Vec<cidr::IpCidr>,
}

impl Machinectl {
    fn convert_machine(&self, machine: &Machine) -> Result<Vec<Record>> {
        let mut records = Vec::new();
        for ip in machine.addresses.split("\n") {
            let ip_addr = std::net::IpAddr::from_str(ip)
                .boxed_local()
                .context(BackendSnafu {
                    backend: BACKEND_NAME,
                    message: format!("Failed to parse ip {}", ip),
                })?;

            // Skip not-included.
            if self.included_cidrs.len() > 0
                && !self
                    .included_cidrs
                    .iter()
                    .any(|included_cidr| included_cidr.contains(&ip_addr))
            {
                continue;
            }

            // Skip excluded. Empty vec handled implicitly.
            if self
                .excluded_cidrs
                .iter()
                .any(|excluded_cidr| excluded_cidr.contains(&ip_addr))
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
                source: BACKEND_NAME.to_string(),
            });
        }

        Ok(records)
    }
}

impl common::Backend for Machinectl {
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

fn convert_cidrs(cidrs_opt: Option<String>) -> Vec<cidr::IpCidr> {
    cidrs_opt
        .unwrap_or_default()
        .split(",")
        .filter_map(|cidr| {
            // Gracefully handle null strings
            if cidr.trim().len() == 0 {
                return None;
            }
            Some(
                cidr::IpCidr::from_str(cidr.trim())
                    .map_err(|err| {
                        ConfigSnafu {
                            message: format!("Invalid CIDR {cidr}: {err}"),
                            prefix: "machinectl",
                        }
                        .build()
                    })
                    .unwrap(),
            )
        })
        .collect()
}

impl From<super::Config> for Machinectl {
    fn from(value: super::Config) -> Self {
        // Unfortunately config-rs makes it difficult to mix
        // strings and vec of strings, so we have to parse ourselves
        Self {
            domain: value.domain,
            excluded_cidrs: convert_cidrs(value.excluded_cidrs),
            included_cidrs: convert_cidrs(value.included_cidrs),
        }
    }
}
