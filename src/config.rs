use crate::common::{Backend, Frontend};

#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub cloudflare: Option<crate::cloudflare::Config>,

    pub headscale: Option<crate::headscale::Config>,
    pub machinectl: Option<crate::machinectl::Config>,

    pub sync: crate::service::Config,
}

impl Config {
    pub fn get_service(self) -> crate::service::DNSSync {
        let sync_config = self.sync.clone();
        let (backends, frontends) = self.into_impls();
        crate::service::DNSSync::new(sync_config, backends, frontends)
    }

    pub fn into_impls(self) -> (Vec<Box<dyn Backend>>, Vec<Box<dyn Frontend>>) {
        let mut backends: Vec<Box<dyn Backend>> = Vec::new();

        if let Some(cfg) = self.cloudflare {
            backends.push(Box::new(crate::cloudflare::CloudflareBackend::from(cfg)));
        }

        let mut frontends: Vec<Box<dyn Frontend>> = Vec::new();

        if let Some(cfg) = self.headscale {
            frontends.push(Box::new(crate::headscale::HeadscaleFrontend::from(cfg)));
        }
        if let Some(cfg) = self.machinectl {
            frontends.push(Box::new(crate::machinectl::MachinectlFrontend::from(cfg)));
        }

        (backends, frontends)
    }
}
