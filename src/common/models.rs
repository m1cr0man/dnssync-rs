pub const RECORD_KIND_A: &str = "A";
pub const RECORD_KIND_AAAA: &str = "AAAA";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Record {
    pub name: url::Host,
    pub kind: String,
    pub content: String,
}

impl Matchable for Record {
    /// Matches returns whether 2 records are of the same name and kind,
    /// it does not check the content of the record.
    fn matches(&self, other: &Self) -> bool {
        return self.name == other.name && self.kind == other.kind;
    }
}

pub trait Frontend {
    fn get_domain(&self) -> url::Host;
    fn set_records(&mut self, records: Vec<Record>, dry_run: bool) -> super::Result<()>;
}

pub trait Backend {
    fn read_records(&self) -> super::Result<Vec<Record>>;
}

pub trait Matchable {
    fn matches(&self, other: &Self) -> bool;
}

pub trait Updateable {
    fn update(self, authority: Record) -> Self;
}
