use std::{fs::File, sync::Arc};

use parking_lot::{ArcRwLockUpgradableReadGuard, ArcRwLockWriteGuard, RawRwLock, RwLock};

use crate::page_inner::PageInner;

#[derive(Debug)]
pub struct Page<T>(Arc<RwLock<PageInner<T>>>);

impl<T> Page<T> {
    pub fn new(file: &File) -> anyhow::Result<Self> {
        Ok(Page(Arc::new(RwLock::new(PageInner::new(file)?))))
    }

    pub fn parse(file: &File) -> anyhow::Result<Self> {
        Ok(Page(Arc::new(RwLock::new(PageInner::parse(file)?))))
    }

    pub fn read(&self) -> ArcRwLockUpgradableReadGuard<RawRwLock, PageInner<T>> {
        self.0.upgradable_read_arc()
    }

    pub fn write(&self) -> ArcRwLockWriteGuard<RawRwLock, PageInner<T>> {
        self.0.write_arc()
    }
}

impl<T> Clone for Page<T> {
    fn clone(&self) -> Self {
        Page(Arc::clone(&self.0))
    }
}
