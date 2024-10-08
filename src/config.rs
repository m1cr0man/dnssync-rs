use crate::common::{Backend, Frontend};
use crate::{cloudflare, headscale, jsonfile, machinectl};

#[cfg(feature = "cli")]
const ENV_PREFIX: &str = "DNSSYNC";

#[derive(Clone, serde::Deserialize)]
pub struct Config {
    backends: Vec<String>,
    frontends: Vec<String>,

    pub cloudflare: Option<cloudflare::Config>,

    pub headscale: Option<headscale::Config>,
    pub jsonfile: Option<jsonfile::Config>,
    pub machinectl: Option<machinectl::Config>,
}

impl Config {
    pub fn get_service(self) -> crate::service::DNSSync {
        let (backends, frontends) = self.into_impls();
        crate::service::DNSSync::new(backends, frontends)
    }

    pub fn into_impls(self) -> (Vec<Box<dyn Backend>>, Vec<Box<dyn Frontend>>) {
        let mut backends: Vec<Box<dyn Backend>> = Vec::new();

        if let Some(cfg) = self.headscale {
            backends.push(Box::new(headscale::Backend::from(cfg)));
            tracing::info!(backend = headscale::BACKEND_NAME, "Loaded backend");
        }
        if let Some(cfg) = self.jsonfile {
            backends.push(Box::new(jsonfile::Backend::from(cfg)));
            tracing::info!(backend = jsonfile::BACKEND_NAME, "Loaded backend");
        }
        if let Some(cfg) = self.machinectl {
            backends.push(Box::new(machinectl::Machinectl::from(cfg)));
            tracing::info!(backend = machinectl::BACKEND_NAME, "Loaded backend");
        }

        let mut frontends: Vec<Box<dyn Frontend>> = Vec::new();

        if let Some(cfg) = self.cloudflare {
            frontends.push(Box::new(cloudflare::Cloudflare::from(cfg)));
            tracing::info!(frontend = cloudflare::FRONTEND_NAME, "Loaded frontend");
        }

        (backends, frontends)
    }

    pub fn with_services(backends: Vec<String>, frontends: Vec<String>) -> Self {
        Self {
            backends,
            frontends,
            cloudflare: None,
            headscale: None,
            machinectl: None,
            jsonfile: None,
        }
    }

    #[cfg(feature = "cli")]
    pub fn populate_from_env(mut self) -> crate::common::Result<Self> {
        use crate::common::ConfigSnafu;

        for imp in self.backends.iter() {
            match imp.to_lowercase().as_str() {
                "headscale" => {
                    self.headscale = Some(parse_config(&format!("{ENV_PREFIX}_HEADSCALE"))?)
                }
                "jsonfile" => {
                    self.jsonfile = Some(parse_config(&format!("{ENV_PREFIX}_JSONFILE"))?)
                }
                "machinectl" => {
                    self.machinectl = Some(parse_config(&format!("{ENV_PREFIX}_MACHINECTL"))?)
                }
                be => {
                    return Err(ConfigSnafu {
                        prefix: format!("backends"),
                        message: format!("Unrecognized backend {be}"),
                    }
                    .build())
                }
            }
        }
        for imp in self.frontends.iter() {
            match imp.to_lowercase().as_str() {
                "cloudflare" => {
                    self.cloudflare = Some(parse_config(&format!("{ENV_PREFIX}_CLOUDFLARE"))?)
                }
                be => {
                    return Err(ConfigSnafu {
                        prefix: format!("frontends"),
                        message: format!("Unrecognized frontend {be}"),
                    }
                    .build())
                }
            }
        }

        Ok(self)
    }
}

#[cfg(feature = "cli")]
fn parse_config<'a, T: serde::Deserialize<'a>>(prefix: &str) -> crate::common::Result<T> {
    let cfg_source = config::Config::builder()
        .add_source(
            config::Environment::with_prefix(prefix)
                .convert_case(config::Case::ScreamingSnake)
                .try_parsing(true),
        )
        .build()
        .map_err(|e| {
            crate::common::ConfigSnafu {
                prefix,
                message: format!("Error reading configuration source: {e}"),
            }
            .build()
        })?;

    cfg_source.try_deserialize().map_err(|e| {
        crate::common::ConfigSnafu {
            prefix,
            message: format!("Error in configuration: {e}"),
        }
        .build()
    })
}
