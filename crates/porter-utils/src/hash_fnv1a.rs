pub trait HashFnv1a {
    /// Creates a fnv1a hash for this data.
    fn hash_fnv1a(&self, offset: u64, prime: u64) -> u64;
}

/// Calculates the 64-bit FNV-1a hash of a byte slice.
fn fnv1a_hash(data: &[u8], offset: u64, prime: u64) -> u64 {
    let mut result = offset;

    for &byte in data {
        result ^= byte as u64;
        result = result.wrapping_mul(prime);
    }

    result
}

impl HashFnv1a for &str {
    fn hash_fnv1a(&self, offset: u64, prime: u64) -> u64 {
        fnv1a_hash(self.as_bytes(), offset, prime)
    }
}

impl HashFnv1a for String {
    fn hash_fnv1a(&self, offset: u64, prime: u64) -> u64 {
        fnv1a_hash(self.as_bytes(), offset, prime)
    }
}

impl HashFnv1a for &[u8] {
    fn hash_fnv1a(&self, offset: u64, prime: u64) -> u64 {
        fnv1a_hash(self, offset, prime)
    }
}
