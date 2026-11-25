//! A small, fast hasher (FNV-1a over the bytes) for the maps built while a level loads. The standard `HashMap` hashes with SipHash, which cost
//! a visible share of the load time for the hundreds of thousands of short keys of a long level; these maps never see attacker-chosen keys.
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default, Clone, Copy)]
pub struct FastHasher(u64);

impl Hasher for FastHasher {
    #[inline]
    fn finish(&self) -> u64 { self.0 }
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut h = if self.0 == 0 { 0xcbf29ce484222325 } else { self.0 };
        for &b in bytes { h = (h ^ b as u64).wrapping_mul(0x100000001b3); }
        self.0 = h;
    }
    #[inline]
    fn write_u8(&mut self, i: u8) { self.write(&[i]); }
    #[inline]
    fn write_u32(&mut self, i: u32) { self.write(&i.to_le_bytes()); }
    #[inline]
    fn write_u64(&mut self, i: u64) { self.write(&i.to_le_bytes()); }
    #[inline]
    fn write_usize(&mut self, i: usize) { self.write(&i.to_le_bytes()); }
}

pub type FastMap<K, V> = HashMap<K, V, BuildHasherDefault<FastHasher>>;
