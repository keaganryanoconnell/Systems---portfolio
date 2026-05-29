use std::collections::HashMap;
use std::hash::Hash;

pub struct LwwSet<V: Clone + Eq + Hash> {
    pub add_set: HashMap<V, u64>,
    pub remove_set: HashMap<V, u64>,
    pub peer_id: u32,
    pub clock: u64,
    pub last_merged_clock: u64,
}

impl<V: Clone + Eq + Hash> LwwSet<V> {
    pub fn new(peer_id: u32) -> Self {
        Self {
            add_set: HashMap::new(),
            remove_set: HashMap::new(),
            peer_id,
            clock: 0,
            last_merged_clock: 0,
        }
    }

    pub fn add(&mut self, value: V) {
        self.clock += 1;
        self.add_set.insert(value, self.clock);
    }

    pub fn remove(&mut self, value: &V) {
        self.clock += 1;
        self.remove_set.insert(value.clone(), self.clock);
    }

    pub fn contains(&self, value: &V) -> bool {
        let add_ts = self.add_set.get(value).copied().unwrap_or(0);
        let remove_ts = self.remove_set.get(value).copied().unwrap_or(0);
        add_ts > remove_ts
    }

    pub fn elements(&self) -> Vec<&V> {
        self.add_set.keys()
            .filter(|v| self.contains(v))
            .collect()
    }

    pub fn merge(&mut self, other: &Self) {
        for (v, ts) in &other.add_set {
            self.add_set.entry(v.clone())
                .and_modify(|t| *t = (*t).max(*ts))
                .or_insert(*ts);
        }
        for (v, ts) in &other.remove_set {
            self.remove_set.entry(v.clone())
                .and_modify(|t| *t = (*t).max(*ts))
                .or_insert(*ts);
        }
        self.clock = self.clock.max(other.clock);
    }

    pub fn compute_delta(&self) -> LwwSet<V> {
        let mut delta = LwwSet::new(self.peer_id);

        for (v, ts) in &self.add_set {
            if *ts > self.last_merged_clock {
                delta.add_set.insert(v.clone(), *ts);
            }
        }
        for (v, ts) in &self.remove_set {
            if *ts > self.last_merged_clock {
                delta.remove_set.insert(v.clone(), *ts);
            }
        }

        delta
    }

    pub fn mark_merged(&mut self) {
        self.last_merged_clock = self.clock;
    }

    pub fn delta_size_bytes(&self) -> usize {
        self.add_set.len() * 42 + self.remove_set.len() * 42
    }

    pub fn total_size_bytes(&self) -> usize {
        (self.add_set.len() + self.remove_set.len()) * 64
    }
}

impl<V: Clone + Eq + Hash> Clone for LwwSet<V> {
    fn clone(&self) -> Self {
        Self {
            add_set: self.add_set.clone(),
            remove_set: self.remove_set.clone(),
            peer_id: self.peer_id,
            clock: self.clock,
            last_merged_clock: self.last_merged_clock,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_contains() {
        let mut set = LwwSet::new(0);
        set.add("key1");
        set.add("key2");
        assert!(set.contains(&"key1"));
        assert!(set.contains(&"key2"));
        assert!(!set.contains(&"key3"));
    }

    #[test]
    fn test_remove() {
        let mut set = LwwSet::new(0);
        set.add("key1");
        set.remove(&"key1");
        assert!(!set.contains(&"key1"));
    }

    #[test]
    fn test_lww_add_wins_over_remove() {
        let mut set = LwwSet::new(0);
        set.remove(&"key1");
        set.add("key1");
        assert!(set.contains(&"key1"));
    }

    #[test]
    fn test_merge_converges() {
        let mut a = LwwSet::new(1);
        let mut b = LwwSet::new(2);

        a.add("shared");
        b.add("shared");
        a.add("a_only");
        b.add("b_only");

        a.merge(&b);
        b.merge(&a);

        assert!(a.contains(&"shared"));
        assert!(a.contains(&"a_only"));
        assert!(a.contains(&"b_only"));
        assert!(b.contains(&"shared"));
        assert!(b.contains(&"a_only"));
        assert!(b.contains(&"b_only"));
    }

    #[test]
    fn test_delta_is_compact() {
        let mut set = LwwSet::new(0);
        set.add("key1");
        set.add("key2");
        set.mark_merged();

        set.add("key3");

        let delta = set.compute_delta();
        assert!(delta.add_set.contains_key("key3"));
        assert!(!delta.add_set.contains_key("key1"));
    }

    #[test]
    fn test_concurrent_merge_idempotent() {
        let mut a = LwwSet::new(1);
        a.add("x");
        let snapshot = a.clone();

        for _ in 0..100 {
            let copy = snapshot.clone();
            a.merge(&copy);
        }

        assert!(a.contains(&"x"));
        assert_eq!(a.elements().len(), 1);
    }

    #[test]
    fn test_elements_returns_only_active() {
        let mut set = LwwSet::new(0);
        set.add("active");
        set.add("removed");
        set.remove(&"removed");

        let elems = set.elements();
        assert!(elems.contains(&&"active"));
        assert!(!elems.contains(&&"removed"));
    }
}
