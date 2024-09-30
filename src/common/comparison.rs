use super::{Manage, Match, Record, Update};

pub(crate) struct DiffResult<R> {
    pub create: Vec<R>,
    pub update: Vec<R>,
    pub delete: Vec<R>,
}

impl<R> DiffResult<R> {
    pub fn len(&self) -> usize {
        return self.create.len() + self.update.len() + self.delete.len();
    }
}

pub(crate) fn diff_records<R: Clone + Manage + Match + Update + PartialEq + From<Record>>(
    current: Vec<R>,
    authority: Vec<Record>,
) -> DiffResult<R> {
    let mut create: Vec<R> = Vec::with_capacity(authority.len());
    let mut update: Vec<R> = Vec::with_capacity(authority.len());
    let mut delete: Vec<R> = Vec::with_capacity(current.len());

    authority.clone().into_iter().for_each(|record| {
        let record_conv: R = record.clone().into();
        match current.iter().find(|r| r.matches(&record_conv)) {
            Some(existing) => {
                // Check for existing unmanaged record
                if !existing.is_managed() {
                    tracing::warn!(
                        name = record.name.to_string(),
                        kind = record.kind,
                        "Skipping update to an unmanaged record"
                    );
                }
                // Only update if content differs
                else if existing != &record_conv {
                    update.push(existing.to_owned().update(record));
                }
            }
            None => create.push(record_conv),
        }
    });

    current
        .into_iter()
        .filter(|record| record.is_managed())
        .for_each(|record| {
            if let None = authority
                .iter()
                .find(|&r| record.matches(&r.to_owned().into()))
            {
                delete.push(record);
            }
        });

    DiffResult {
        create,
        update,
        delete,
    }
}
