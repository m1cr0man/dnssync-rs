use crate::common::{Backend, Frontend, Match, Record, Result};

type Backends = Vec<Box<dyn Backend>>;
type Frontends = Vec<Box<dyn Frontend>>;

pub struct DNSSync {
    backends: Backends,
    frontends: Frontends,
}

impl DNSSync {
    pub fn new(backends: Backends, mut frontends: Frontends) -> Self {
        // Sort the frontends by length descending. This will guarantee
        // during sync that records are paired with the longest matching suffix.
        frontends.sort_by_key(|fe| fe.get_domain().len());
        frontends.reverse();
        Self {
            backends,
            frontends,
        }
    }

    pub fn sync(&mut self, dry_run: bool) -> Result<()> {
        // Build a list of all records
        let mut authority: Vec<Record> = Vec::new();
        for backend in self.backends.iter() {
            authority.extend(backend.read_records()?.into_iter());
        }

        // Check for duplicates
        let num_records = authority.len();
        for (i, record) in authority.iter().enumerate() {
            authority
                .split_at((i + 1).min(num_records))
                .1
                .into_iter()
                .for_each(|dupe| {
                    if record.matches(dupe) {
                        tracing::warn!(
                            name = record.name.to_string(),
                            kind = record.kind,
                            backend = record.source,
                            other_backend = dupe.source,
                            value = record.content,
                            other_value = dupe.content,
                            concat!(
                                "Duplicate record. This may result in",
                                " an unstable diff between runs."
                            )
                        );
                    }
                });
        }

        // Now that authority is established, we map each record to a frontend.
        // Use frontend's vec index as a key.
        let mut paired: Vec<Vec<Record>> = Vec::with_capacity(self.frontends.len());
        paired.resize_with(self.frontends.len(), || Vec::with_capacity(authority.len()));

        for record in authority {
            let domain = record.name.to_string();
            match self
                .frontends
                .iter()
                .enumerate()
                .find(|(_, fe)| domain.ends_with(&fe.get_domain()))
            {
                Some((i, _)) => {
                    // Extend an entry in the authority map
                    paired
                        .get_mut(i)
                        .expect("Frontend must exist for pairing to be generated")
                        .push(record);
                }
                None => tracing::warn!(
                    name = record.name.to_string(),
                    kind = record.kind,
                    backend = record.source,
                    "No frontends map to this record"
                ),
            };
        }

        // Provide the authoritative list of records to each fronend
        for (i, records) in paired.into_iter().enumerate() {
            let frontend = self
                .frontends
                .get_mut(i)
                .expect("Frontend must exist for pairing to be generated");
            frontend.set_records(records, dry_run)?;
        }

        Ok(())
    }
}
