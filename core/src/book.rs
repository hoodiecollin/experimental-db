use std::sync::Arc;

use parking_lot::{ArcRwLockUpgradableReadGuard, ArcRwLockWriteGuard, RawRwLock, RwLock};

use crate::{book_inner::BookInner, BookId};

#[derive(Debug)]
pub struct Book<T>(Arc<RwLock<BookInner<T>>>);

impl<T> Clone for Book<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> Book<T> {
    pub fn new(id: BookId) -> anyhow::Result<Self> {
        Ok(Book(Arc::new(RwLock::new(BookInner::new(id)?))))
    }

    pub fn read(&self) -> ArcRwLockUpgradableReadGuard<RawRwLock, BookInner<T>> {
        self.0.upgradable_read_arc()
    }

    pub fn write(&self) -> ArcRwLockWriteGuard<RawRwLock, BookInner<T>> {
        self.0.write_arc()
    }
}
