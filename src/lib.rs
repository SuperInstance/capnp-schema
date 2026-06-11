#[derive(Debug, Clone)]
pub struct CapnpField {
    pub name: String,
    pub slot_type: String,
    pub ordinal: u16,
}

#[derive(Debug)]
pub struct CapnpStruct {
    pub name: String,
    pub data_word_count: u16,
    pub pointer_count: u16,
    pub fields: Vec<CapnpField>,
}

impl CapnpStruct {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), data_word_count: 0, pointer_count: 0, fields: Vec::new() }
    }

    pub fn total_size_bytes(&self) -> usize {
        (self.data_word_count as usize + self.pointer_count as usize) * 8
    }
}

pub fn parse_capnp_id(line: &str) -> Option<u64> {
    line.split('@').nth(1)?.trim().parse().ok()
}
