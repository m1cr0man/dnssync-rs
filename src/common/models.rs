pub const RECORD_KIND_A: &str = "A";
pub const RECORD_KIND_AAAA: &str = "AAAA";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Record {
    pub name: url::Host,
    pub kind: String,
    pub content: String,
}

impl Match for Record {
    /// Matches returns whether 2 records are of the same name and kind,
    /// it does not check the content of the record.
    fn matches(&self, other: &Self) -> bool {
        self.kind.eq_ignore_ascii_case(&other.kind)
            && self
                .name
                .to_string()
                .eq_ignore_ascii_case(&other.name.to_string())
    }
}

pub trait Frontend {
    fn get_domain(&self) -> String;
    fn set_records(
        &mut self,
        domain: String,
        records: Vec<Record>,
        dry_run: bool,
    ) -> super::Result<()>;
}

pub trait Backend {
    fn get_domain(&self) -> String;
    fn read_records(&self) -> super::Result<Vec<Record>>;
}

pub trait Match {
    fn matches(&self, other: &Self) -> bool;
}

pub trait Update {
    fn update(self, authority: Record) -> Self;
}

pub trait Manage {
    fn is_managed(&self) -> bool;
}
