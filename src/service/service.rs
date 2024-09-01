use crate::common::{Backend, Frontend, Result};

pub struct DNSSync<B: Backend, F: Frontend> {
    config: super::Config,
    backend: B,
    frontend: F,
}

impl<B: Backend, F: Frontend> DNSSync<B, F> {
    pub fn new(config: super::Config, backend: B, frontend: F) -> Self {
        Self {
            config,
            backend,
            frontend,
        }
    }

    pub fn sync(&mut self) -> Result<()> {
        let authority = self.frontend.read_records()?;
        let state: Vec<crate::common::Record> = self.backend.read_records()?;
        let mut delete = Vec::new();

        for record in state {
            if let None = authority.iter().find(|other| record.matches(other)) {
                delete.push(record);
            }
        }

        self.backend.write_records(authority)?;
        self.backend.delete_records(delete)?;

        Ok(())
    }
}
