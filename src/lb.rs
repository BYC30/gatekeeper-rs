use crate::config::{KeyConfig, LbPolicy};
use crate::errors::Result;
use crate::keypool::{Key, KeyPool};

#[derive(Debug)]
pub struct LoadBalancer {
    policy: LbPolicy,
    pool: KeyPool,
}

impl LoadBalancer {
    pub fn new(policy: LbPolicy, keys: &[KeyConfig]) -> Self {
        Self {
            policy,
            pool: KeyPool::new(keys),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }

    pub fn select(&self) -> Result<&Key> {
        match self.policy {
            LbPolicy::RoundRobin => self.pool.pick_rr(),
            LbPolicy::WeightedRoundRobin => self.pool.pick_rr(),
            LbPolicy::LeastPending => self.pool.pick_rr(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfgs() -> Vec<KeyConfig> {
        vec![
            KeyConfig { id: "k1".into(), value: "a".into(), provider: "p".into(), weight: 1 },
            KeyConfig { id: "k2".into(), value: "b".into(), provider: "p".into(), weight: 1 },
        ]
    }

    #[test]
    fn rr_select() {
        let lb = LoadBalancer::new(LbPolicy::RoundRobin, &cfgs());
        let k1 = lb.select().unwrap().id.clone();
        let k2 = lb.select().unwrap().id.clone();
        let k3 = lb.select().unwrap().id.clone();
        assert_eq!(k1, "k1");
        assert_eq!(k2, "k2");
        assert_eq!(k3, "k1");
    }
}
