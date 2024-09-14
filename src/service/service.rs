use crate::common::{Backend, Frontend, Record, Result, SyncSnafu};

type Backends = Vec<Box<dyn Backend>>;
type Frontends = Vec<Box<dyn Frontend>>;

pub struct DNSSync {
    backends: Backends,
    frontends: Frontends,
}

impl DNSSync {
    pub fn new(backends: Backends, frontends: Frontends) -> Self {
        Self {
            backends,
            frontends,
        }
    }

    pub fn sync(&mut self) -> Result<()> {
        let mut domains: Vec<url::Host> = self.frontends.iter().map(|b| b.get_domain()).collect();

        // Sort lexically. Longest domain prefixes will be first.
        domains.sort();

        // Map each record to a backend by the longest matching domain.
        let mut authority: Vec<(url::Host, Record)> = Vec::new();
        for backend in self.backends.iter_mut() {
            for record in backend.read_records()? {
                match domains
                    .iter()
                    .find(|d| record.name.to_string().contains(&d.to_string()))
                {
                    Some(domain) => authority.push((domain.to_owned(), record)),
                    None => {
                        return SyncSnafu {
                            message: format!(
                                "No matching backend domain for record {}",
                                record.name
                            ),
                        }
                        .fail()
                    }
                }
            }
        }

        for frontend in self.frontends.iter_mut() {
            let domain = &frontend.get_domain();
            let auth_records = authority
                .clone()
                .into_iter()
                .filter_map(|(dom, record)| dom.eq(domain).then_some(record))
                .collect();

            frontend.set_records(auth_records)?;
        }

        Ok(())
    }
}
