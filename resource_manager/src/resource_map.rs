use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use retgui_runtime::sync::watch;

pub(crate) struct ResourceMap<K, V> {
    resources: watch::Sender<HashMap<K, Arc<V>>>,
}

impl<K: Eq + Hash, V> ResourceMap<K, V> {
    pub(crate) fn new() -> Self {
        Self {
            resources: watch::Sender::new(HashMap::new()),
        }
    }

    pub(crate) fn insert(&self, key: K, value: Arc<V>) {
        let mut replaced = None;
        self.resources.send_modify(|resources| {
            replaced = resources.insert(key, value);
        });
        drop(replaced);
    }

    pub(crate) fn get(&self, key: &K) -> Option<Arc<V>> {
        self.resources.borrow().get(key).cloned()
    }

    pub(crate) fn contains(&self, key: &K) -> bool {
        self.resources.borrow().contains_key(key)
    }
}
