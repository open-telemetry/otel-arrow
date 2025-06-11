use crate::expression::Hasher;

#[derive(Debug, Clone)]
pub struct JsonValueData {
    raw_value: String,
}

impl JsonValueData {
    pub fn get_raw_value(&self) -> &str {
        &self.raw_value
    }

    pub(crate) fn add_hash_bytes(&self, hasher: &mut Hasher) {
        hasher.add_bytes(self.get_raw_value().as_bytes());
    }
}
