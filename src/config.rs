use crate::common::{Backend, Frontend};

#[derive(Clone, serde::Deserialize)]
pub struct Config {
    pub cloudflare: Option<crate::cloudflare::Config>,

    pub headscale: Option<crate::headscale::Config>,
    pub machinectl: Option<crate::machinectl::Config>,
}

impl Config {
    pub fn get_service(self) -> crate::service::DNSSync {
        let (backends, frontends) = self.into_impls();
        crate::service::DNSSync::new(backends, frontends)
    }

    pub fn into_impls(self) -> (Vec<Box<dyn Backend>>, Vec<Box<dyn Frontend>>) {
        let mut backends: Vec<Box<dyn Backend>> = Vec::new();

        if let Some(cfg) = self.headscale {
            backends.push(Box::new(crate::headscale::HeadscaleBackend::from(cfg)));
        }
        if let Some(cfg) = self.machinectl {
            backends.push(Box::new(crate::machinectl::MachinectlBackend::from(cfg)));
        }

        let mut frontends: Vec<Box<dyn Frontend>> = Vec::new();

        if let Some(cfg) = self.cloudflare {
            frontends.push(Box::new(crate::cloudflare::CloudflareFrontend::from(cfg)));
        }

        (backends, frontends)
    }
}
