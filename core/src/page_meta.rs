use std::collections::{BTreeSet, HashMap};

use crate::{page_layout::PageLayout, Idx, IdxOrKey, Key};

#[derive(Debug, Clone, PartialEq)]
pub struct PageMeta<T> {
    layout: PageLayout<T>,
    idx_to_key: HashMap<Idx, Key>,
    key_to_idx: HashMap<Key, Idx>,
    vacant_idx: BTreeSet<Idx>,
}

impl<T> PageMeta<T> {
    pub fn new() -> Self {
        let layout = PageLayout::new();
        let cap = layout.cap;

        PageMeta {
            layout,
            idx_to_key: HashMap::with_capacity(cap),
            key_to_idx: HashMap::with_capacity(cap),
            vacant_idx: (0..cap).map(|n| Idx::new(n as u32)).collect(),
        }
    }

    pub fn parse(file_content: &[u8]) -> anyhow::Result<Self> {
        let layout = PageLayout::new();
        let cap = layout.cap;

        let mut idx_to_key = HashMap::with_capacity(cap);
        let mut key_to_idx = HashMap::with_capacity(cap);
        let mut vacant_idx = BTreeSet::new();

        for (idx, key) in unsafe { layout.page_entry_iter(file_content)? } {
            if let Some(key) = key {
                idx_to_key.insert(idx, key);
                key_to_idx.insert(key, idx);
            } else {
                vacant_idx.insert(idx);
            }
        }

        Ok(PageMeta {
            layout,
            idx_to_key,
            key_to_idx,
            vacant_idx,
        })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.idx_to_key.len()
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.len() == self.layout.cap
    }

    #[inline]
    pub fn is_idx_vacant(&self, idx: Idx) -> bool {
        self.vacant_idx.contains(&idx)
    }

    #[inline]
    pub fn has_key(&self, key: Key) -> bool {
        self.key_to_idx.contains_key(&key)
    }

    #[inline]
    pub fn lookup_key(&self, idx: Idx) -> Option<Key> {
        self.idx_to_key.get(&idx).copied()
    }

    #[inline]
    pub fn lookup_idx(&self, key: Key) -> Option<Idx> {
        self.key_to_idx.get(&key).copied()
    }

    #[inline]
    pub fn vacate(&mut self, idx_or_key: IdxOrKey) -> anyhow::Result<(Idx, Key)> {
        match idx_or_key {
            IdxOrKey::Idx(idx) => {
                if !self.idx_to_key.contains_key(&idx) {
                    anyhow::bail!("Slot is already vacant");
                }

                let key = self.idx_to_key.remove(&idx).expect("Key not found");
                self.key_to_idx.remove(&key).expect("Index not found");
                self.vacant_idx.insert(idx);

                Ok((idx, key))
            }
            IdxOrKey::Key(key) => {
                if !self.key_to_idx.contains_key(&key) {
                    anyhow::bail!("Key not found");
                }

                let idx = self.key_to_idx.remove(&key).expect("Index not found");
                self.idx_to_key.remove(&idx).expect("Key not found");
                self.vacant_idx.insert(idx);

                Ok((idx, key))
            }
        }
    }

    #[inline]
    pub fn insert_idx_and_key(&mut self, idx: Idx, key: Key) -> anyhow::Result<()> {
        if !self.is_idx_vacant(idx) {
            anyhow::bail!("Slot is already occupied");
        }

        unsafe { self.insert_idx_and_key_unchecked(idx, key) };

        Ok(())
    }

    #[inline(always)]
    pub unsafe fn insert_idx_and_key_unchecked(&mut self, idx: Idx, key: Key) {
        self.idx_to_key.insert(idx, key);
        self.key_to_idx.insert(key, idx);
        self.vacant_idx.remove(&idx);
    }

    #[inline]
    pub fn insert_key(&mut self, key: Key) -> anyhow::Result<Idx> {
        if self.has_key(key) {
            anyhow::bail!("Key already exists");
        }

        let idx = if let Some(idx) = self.vacant_idx.iter().next().copied() {
            idx
        } else {
            anyhow::bail!("No more vacant slots");
        };

        self.vacant_idx.remove(&idx);

        unsafe { self.insert_idx_and_key_unchecked(idx, key) };

        Ok(idx)
    }

    #[inline]
    pub fn replace_key(&mut self, idx: Idx, key: Key) -> anyhow::Result<Key> {
        if self.is_idx_vacant(idx) {
            anyhow::bail!("Slot is vacant");
        }

        let old_key = self.idx_to_key.insert(idx, key).expect("Key not found");
        self.key_to_idx.insert(key, idx);
        self.key_to_idx.remove(&old_key).expect("Key not found");

        Ok(old_key)
    }

    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        self.key_to_idx.keys()
    }
}

impl<T> std::ops::Deref for PageMeta<T> {
    type Target = PageLayout<T>;

    fn deref(&self) -> &Self::Target {
        &self.layout
    }
}
