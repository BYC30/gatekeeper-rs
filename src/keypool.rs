use crate::config::KeyConfig;
use crate::errors::Result;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Debug)]
pub struct Key {
    pub id: String,
    pub value: String,
    pub provider: String,
    pub weight: u32,
    pub fail_count: u32,
    pub in_flight: u32,
}

impl From<&KeyConfig> for Key {
    fn from(k: &KeyConfig) -> Self {
        Self {
            id: k.id.clone(),
            value: k.value.clone(),
            provider: k.provider.clone(),
            weight: k.weight,
            fail_count: 0,
            in_flight: 0,
        }
    }
}

#[derive(Debug)]
pub struct KeyPool {
    keys: Vec<Key>,
    rr_idx: AtomicUsize,
}

impl KeyPool {
    pub fn new(configs: &[KeyConfig]) -> Self {
        let keys = configs.iter().map(|k| k.into()).collect();
        Self {
            keys,
            rr_idx: AtomicUsize::new(0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn keys(&self) -> &[Key] {
        &self.keys
    }

    pub fn pick_rr(&self) -> Result<&Key> {
        if self.keys.is_empty() {
            return Err(crate::errors::GatekeeperError::NoAvailableKeys);
        }
        let idx = self.rr_idx.fetch_add(1, Ordering::Relaxed) % self.keys.len();
        Ok(&self.keys[idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfgs() -> Vec<KeyConfig> {
        vec![
            KeyConfig { id: "k1".into(), value: "a".into(), provider: "p".into(), weight: 1 },
            KeyConfig { id: "k2".into(), value: "b".into(), provider: "p".into(), weight: 1 },
            KeyConfig { id: "k3".into(), value: "c".into(), provider: "p".into(), weight: 1 },
        ]
    }

    #[test]
    fn round_robin_cycles() {
        let pool = KeyPool::new(&cfgs());
        let first = pool.pick_rr().unwrap().id.clone();
        let second = pool.pick_rr().unwrap().id.clone();
        let third = pool.pick_rr().unwrap().id.clone();
        let fourth = pool.pick_rr().unwrap().id.clone();
        assert_eq!(first, "k1");
        assert_eq!(second, "k2");
        assert_eq!(third, "k3");
        assert_eq!(fourth, "k1");
    }
}
