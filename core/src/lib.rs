use std::{
    alloc::{alloc, GlobalAlloc, Layout, System},
    mem::MaybeUninit,
    ptr::NonNull,
    sync::Arc,
};

use hashbrown::{HashMap, HashSet};
use memmap2::MmapMut;
use parking_lot::RwLock;
use petgraph::prelude::UnGraphMap;
use serde::{Deserialize, Serialize};

pub type Id = u32;

pub fn new_id() -> Id {
    rand::random()
}

pub type Relationships<T> = UnGraphMap<Id, T>;

type ValuePtr<T> = NonNull<MaybeUninit<T>>;

pub struct Value<T> {
    arc: Arc<RwLock<ValuePtr<T>>>,
}

impl Value<T> {
    pub fn new(value: *mut MaybeUninit<T>) -> Self {
        Value {
            arc: Arc::new(RwLock::new(NonNull::new(value).unwrap())),
        }
    }
}

impl<T> Clone for Value<T> {
    fn clone(&self) -> Self {
        Value {
            arc: Arc::clone(&self.arc),
        }
    }
}

impl<T> std::ops::Deref for Value<T> {
    type Target = Arc<RwLock<ValuePtr<T>>>;

    fn deref(&self) -> &Self::Target {
        &self.arc
    }
}

impl<T> std::ops::DerefMut for Value<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.arc
    }
}

impl<T> AsRef<Arc<RwLock<ValuePtr<T>>>> for Value<T> {
    fn as_ref(&self) -> &Arc<RwLock<ValuePtr<T>>> {
        &self.arc
    }
}

impl<T> AsMut<Arc<RwLock<ValuePtr<T>>>> for Value<T> {
    fn as_mut(&mut self) -> &mut Arc<RwLock<ValuePtr<T>>> {
        &mut self.arc
    }
}

#[derive(Debug)]
pub struct KvPairs<T> {
    map: HashMap<Id, Value<T>>,
}

impl<T> KvPairs<T> {
    pub fn new() -> Self {
        KvPairs {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: Id, val: T) {
        self.map.insert(key, val);
    }

    pub fn remove(&mut self, key: Id) {
        self.map.remove(&key);
    }

    pub fn get(&self, key: Id) -> Option<&T> {
        self.map.get(&key)
    }
}

#[macro_export]
macro_rules! optional {
    ($name:expr) => {
        Some(($name).into())
    };
}
