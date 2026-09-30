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

/// FNV-1a 64 — the digest every substrate in the SuperInstance fleet agrees on.
pub const FNV_OFFSET: u64 = 0xcbf29ce484222325;
pub const FNV_PRIME: u64 = 0x100000001b3;

#[inline]
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h = FNV_OFFSET;
    for &b in bytes {
        h = (h ^ b as u64).wrapping_mul(FNV_PRIME);
    }
    h
}

/// True if this crate's FNV-1a still agrees with the rest of the fleet.
pub fn canary_holds() -> bool {
    fnv1a64("café Δ 日本語".as_bytes()) == 0x024a555471370b18d
}
