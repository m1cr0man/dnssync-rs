pub const RECORD_KIND_A: &str = "A";
pub const RECORD_KIND_AAAA: &str = "AAAA";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Record {
    pub name: url::Host,
    pub kind: String,
    pub content: String,
}

impl Record {
    pub fn matches(&self, other: &Self) -> bool {
        return self.name == other.name && self.kind == other.kind;
    }
}

pub trait Backend {
    fn read_records(&mut self) -> super::Result<Vec<Record>>;
    fn write_records(&mut self, records: Vec<Record>) -> super::Result<()>;
    fn delete_records(&mut self, records: Vec<Record>) -> super::Result<()>;
}

pub trait Frontend {
    fn read_records(&self) -> super::Result<Vec<Record>>;
}
