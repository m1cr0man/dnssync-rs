use std::str::FromStr;

use super::models::{Machine, Machines};
use crate::common::{Frontend, FrontendSnafu, Record, Result, RECORD_KIND_A, RECORD_KIND_AAAA};
use snafu::ResultExt;

pub struct MachinectlFrontend {
    ignored_cidrs: Vec<cidr::IpCidr>,
}

impl MachinectlFrontend {
    fn convert_machine(&self, machine: &Machine) -> Result<Vec<Record>> {
        let mut records = Vec::new();
        for ip in machine.addresses.split("\n") {
            let ip_addr = std::net::IpAddr::from_str(ip)
                .boxed_local()
                .context(FrontendSnafu {
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
                name: machine.name.clone(),
                kind: kind.to_string(),
                content: ip.to_string(),
            });
        }

        Ok(records)
    }
}

impl Frontend for MachinectlFrontend {
    fn read_records(&self) -> Result<Vec<Record>> {
        let output = std::process::Command::new("machinectl")
            .args(["list", "-o", "json"])
            .output()
            .boxed_local()
            .context(FrontendSnafu {
                message: "Failed to run machinectl list",
            })?;

        let data: Machines =
            serde_json::from_str(&String::from_utf8(output.stdout).boxed_local().context(
                FrontendSnafu {
                    message: "Failed to decode stdout",
                },
            )?)
            .boxed_local()
            .context(FrontendSnafu {
                message: "Failed to parse machinectl list output",
            })?;

        let mut records = Vec::new();
        for machine in data {
            records.extend(self.convert_machine(&machine)?);
        }

        Ok(records)
    }
}

impl From<super::Config> for MachinectlFrontend {
    fn from(value: super::Config) -> Self {
        Self {
            ignored_cidrs: value.ignored_cidrs.unwrap_or(Vec::new()),
        }
    }
}
