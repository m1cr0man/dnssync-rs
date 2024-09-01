pub use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct WrappedRecord {
    pub ttl: usize,
    pub record: crate::common::Record,
    pub tracing_id: Uuid,
}
