use crate::common::{Backend, Frontend, Record, Result, SyncSnafu};

type Backends = Vec<Box<dyn Backend>>;
type Frontends = Vec<Box<dyn Frontend>>;

pub struct DNSSync {
    config: super::Config,
    backends: Backends,
    frontends: Frontends,
}

impl DNSSync {
    pub fn new(config: super::Config, backends: Backends, frontends: Frontends) -> Self {
        Self {
            config,
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
        for s in self.backends.iter_mut() {
            for record in s.read_records()? {
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

        for s in self.frontends.iter_mut() {
            let domain = &s.get_domain();
            let mut auth_records = authority
                .iter()
                .filter_map(|(dom, record)| dom.eq(domain).then_some(record));

            let state_records = s.read_records()?;
            let changes: Vec<Record> = auth_records
                .clone()
                .filter_map(|record| {
                    state_records
                        .iter()
                        .any(|sr| sr.matches(record) && sr.content != record.content)
                        .then(|| record.to_owned())
                })
                .collect();
            let deletes: Vec<Record> = state_records
                .iter()
                .filter_map(|record| {
                    (!auth_records.any(|ar| ar.matches(record))).then(|| record.to_owned())
                })
                .collect();

            s.write_records(changes)?;
            s.delete_records(deletes)?;
        }

        Ok(())
    }
}
