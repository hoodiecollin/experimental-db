use std::{path::PathBuf, sync::LazyLock};

use petgraph::prelude::UnGraphMap;

pub mod book;
pub mod book_inner;
pub mod page;
pub mod page_entry;
pub mod page_inner;
pub mod page_layout;
pub mod page_meta;

#[cfg(test)]
mod tests;

#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Idx {
    pub val: u32,
}

impl Idx {
    #[inline]
    pub fn new(val: u32) -> Self {
        Self { val }
    }

    #[inline]
    pub fn as_usize(&self) -> usize {
        self.val as usize
    }
}

impl std::fmt::Debug for Idx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Idx({})", self.val)
    }
}

impl std::fmt::Display for Idx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.val)
    }
}

#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key {
    pub val: u32,
}

impl Key {
    #[inline]
    pub fn new(val: u32) -> Self {
        Self { val }
    }

    #[inline]
    pub fn rand() -> Self {
        Self {
            val: rand::random(),
        }
    }
}

impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Key({:#010x})", self.val)
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#010x}", self.val)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdxOrKey {
    Idx(Idx),
    Key(Key),
}

impl std::fmt::Debug for IdxOrKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdxOrKey::Idx(idx) => write!(f, "IdxOrKey::Idx({})", idx.val),
            IdxOrKey::Key(key) => write!(f, "IdxOrKey::Key({:#010x})", key.val),
        }
    }
}

pub type Relationships<T> = UnGraphMap<Key, T>;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BookId {
    pub val: u64,
}

impl BookId {
    #[inline]
    pub fn new(val: u64) -> Self {
        Self { val }
    }

    #[inline]
    pub fn rand() -> Self {
        Self {
            val: rand::random(),
        }
    }
}

pub static DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let home = dirs::home_dir().expect("No home directory found");
    home.join(".experimental-db")
});
