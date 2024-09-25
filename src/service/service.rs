use std::collections::HashMap;

use crate::common::{Backend, Frontend, Record, Result, SyncSnafu};

type Backends = Vec<Box<dyn Backend>>;
type Frontends = Vec<Box<dyn Frontend>>;

pub struct DNSSync {
    backends: Backends,
    frontends: Frontends,
}

impl DNSSync {
    pub fn new(backends: Backends, mut frontends: Frontends) -> Self {
        // Sort lexically. Longest domain prefixes will be first.
        frontends.sort_by_key(|f| f.get_domain());
        Self {
            backends,
            frontends,
        }
    }

    pub fn sync(&mut self, dry_run: bool) -> Result<()> {
        // Build a mapping of domains to a list of records they should contain.
        let mut authority: HashMap<String, Vec<Record>> = HashMap::new();
        for backend in self.backends.iter_mut() {
            let domain = backend.get_domain();
            let records = backend.read_records()?;

            // Create an entry in the authority map, or extend an existing one.
            match authority.get_mut(&domain) {
                Some(auth) => {
                    auth.extend(records.into_iter());
                }
                None => {
                    authority.insert(domain.to_owned(), records);
                }
            }
        }

        // Now that authority is established, we map each domain to one frontend.
        for (domain, records) in authority {
            match self
                .frontends
                .iter_mut()
                .find(|fe| domain.ends_with(&fe.get_domain()))
            {
                Some(fe) => {
                    tracing::debug!(
                        backend_domain = domain,
                        frontend_domain = fe.get_domain(),
                        "Domain mapped"
                    );
                    fe.set_records(domain, records, dry_run)?;
                }
                None => {
                    return SyncSnafu {
                        message: format!("Could not map domain {domain} to any frontend"),
                    }
                    .fail();
                }
            }
        }

        Ok(())
    }
}
