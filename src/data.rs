use std::collections::{BTreeMap, HashMap, HashSet};

use qp_trie::Trie;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Collection sizes, dense around the typical linear-vs-hash crossover.
pub const SIZES: &[usize] = &[
    1, 2, 4, 8, 16, 24, 32, 48, 64, 96, 128, 256, 512, 1024, 2048, 4096,
];

pub const STRING_LEN: usize = 16;
const SEED: u64 = 0xC0FFEE_u64;
const ALPHANUM: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

/// A missing string that cannot collide with generated alphanumeric keys.
pub const MISSING_STRING: &str = "________________";

pub struct IntData {
    pub keys: Vec<u64>,
    pub missing: u64,
    pub missing_keys: Vec<u64>,
}

pub struct StringData {
    pub keys: Vec<String>,
    pub missing: String,
    pub missing_keys: Vec<String>,
}

impl IntData {
    pub fn generate(n: usize) -> Self {
        let mut rng = StdRng::seed_from_u64(SEED ^ n as u64);
        let count = (n * 2).max(2);
        let mut set = HashSet::with_capacity(count);
        while set.len() < count {
            set.insert(rng.random::<u64>());
        }
        let all: Vec<u64> = set.into_iter().collect();
        let (keys_slice, missing_slice) = all.split_at(n);
        let keys = keys_slice.to_vec();
        let missing_keys = missing_slice.to_vec();
        let missing = missing_keys[0];
        debug_assert_eq!(keys.len(), n);
        Self {
            keys,
            missing,
            missing_keys,
        }
    }

    pub fn mid_key(&self) -> u64 {
        self.keys[self.keys.len() / 2]
    }
}

impl StringData {
    pub fn generate(n: usize) -> Self {
        let mut rng = StdRng::seed_from_u64(SEED ^ 0x9E37 ^ n as u64);
        let mut set = HashSet::with_capacity(n);
        while set.len() < n {
            set.insert(random_string(&mut rng, STRING_LEN));
        }
        let keys: Vec<String> = set.into_iter().collect();

        let count = n.max(1);
        let mut missing_set = HashSet::with_capacity(count);
        let keys_set: HashSet<&str> = keys.iter().map(|s| s.as_str()).collect();
        while missing_set.len() < count {
            let s = random_string(&mut rng, STRING_LEN);
            if !keys_set.contains(s.as_str()) {
                missing_set.insert(s);
            }
        }
        let missing_keys: Vec<String> = missing_set.into_iter().collect();

        Self {
            keys,
            missing: MISSING_STRING.to_owned(),
            missing_keys,
        }
    }

    pub fn mid_key(&self) -> &str {
        &self.keys[self.keys.len() / 2]
    }
}

fn random_string(rng: &mut impl Rng, len: usize) -> String {
    (0..len)
        .map(|_| {
            let i = rng.random_range(0..ALPHANUM.len());
            ALPHANUM[i] as char
        })
        .collect()
}

pub fn int_hashmap(keys: &[u64]) -> HashMap<u64, ()> {
    keys.iter().copied().map(|k| (k, ())).collect()
}

pub fn int_btreemap(keys: &[u64]) -> BTreeMap<u64, ()> {
    keys.iter().copied().map(|k| (k, ())).collect()
}

pub fn string_hashmap(keys: &[String]) -> HashMap<String, ()> {
    keys.iter().cloned().map(|k| (k, ())).collect()
}

pub fn string_btreemap(keys: &[String]) -> BTreeMap<String, ()> {
    keys.iter().cloned().map(|k| (k, ())).collect()
}

pub fn string_trie(keys: &[String]) -> Trie<qp_trie::wrapper::BString, ()> {
    let mut trie = Trie::new();
    for k in keys {
        trie.insert_str(k, ());
    }
    trie
}

#[inline]
pub fn linear_contains_u64(keys: &[u64], needle: u64) -> bool {
    keys.contains(&needle)
}

#[inline]
pub fn linear_contains_str(keys: &[String], needle: &str) -> bool {
    keys.iter().any(|s| s == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_structures_agree() {
        let data = IntData::generate(128);
        let map = int_hashmap(&data.keys);
        let tree = int_btreemap(&data.keys);

        for k in data.keys.iter().take(16) {
            assert!(linear_contains_u64(&data.keys, *k));
            assert!(map.contains_key(k));
            assert!(tree.contains_key(k));
        }
        assert!(!linear_contains_u64(&data.keys, data.missing));
        assert!(!map.contains_key(&data.missing));
        assert!(!tree.contains_key(&data.missing));
        assert_eq!(data.missing_keys.len(), 128);
        for k in &data.missing_keys {
            assert!(!linear_contains_u64(&data.keys, *k));
            assert!(!map.contains_key(k));
            assert!(!tree.contains_key(k));
        }
    }

    #[test]
    fn string_structures_agree() {
        let data = StringData::generate(64);
        let map = string_hashmap(&data.keys);
        let tree = string_btreemap(&data.keys);
        let trie = string_trie(&data.keys);

        for k in data.keys.iter().take(16) {
            assert!(linear_contains_str(&data.keys, k));
            assert!(map.contains_key(k));
            assert!(tree.contains_key(k));
            assert!(trie.contains_key_str(k));
        }
        assert!(!linear_contains_str(&data.keys, &data.missing));
        assert!(!map.contains_key(&data.missing));
        assert!(!tree.contains_key(&data.missing));
        assert!(!trie.contains_key_str(&data.missing));
        assert_eq!(data.missing.len(), STRING_LEN);
        assert_eq!(data.missing_keys.len(), 64);
        for k in &data.missing_keys {
            assert!(!linear_contains_str(&data.keys, k));
            assert!(!map.contains_key(k));
            assert!(!tree.contains_key(k));
            assert!(!trie.contains_key_str(k));
        }
    }

    #[test]
    fn mid_key_is_present() {
        for &n in &[1usize, 2, 3, 16] {
            let ints = IntData::generate(n);
            assert!(ints.keys.contains(&ints.mid_key()));
            let strings = StringData::generate(n);
            assert!(strings.keys.iter().any(|s| s == strings.mid_key()));
        }
    }
}
