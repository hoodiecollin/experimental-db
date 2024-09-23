use std::fs::File;

use memmap2::MmapMut;

use crate::{
    page_entry::{PageEntryMut, PageEntryRef},
    page_layout::PAGE_SIZE,
    page_meta::PageMeta,
    Idx, IdxOrKey, Key,
};

#[derive(Debug)]
pub struct PageInner<T> {
    data: MmapMut,
    meta: PageMeta<T>,
}

impl<T> PageInner<T> {
    /// Create a new empty `PageInner`.
    pub fn new(file: &File) -> anyhow::Result<Self> {
        file.set_len(PAGE_SIZE as u64)?;

        let mut data = unsafe { MmapMut::map_mut(file)? };
        let meta = PageMeta::new();

        // note: ensure the bitmap is zeroed
        unsafe {
            std::ptr::write_bytes(data.as_mut_ptr(), 0, meta.bitmap_bytes);
        }

        Ok(PageInner { data, meta })
    }

    /// Parse an existing `PageInner`.
    pub fn parse(file: &File) -> anyhow::Result<Self> {
        let data = unsafe { MmapMut::map_mut(file) }?;
        let meta = PageMeta::parse(data.as_ref())?;

        Ok(PageInner { data, meta })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.meta.len()
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.meta.is_full()
    }

    #[inline]
    pub fn is_idx_vacant(&self, idx: Idx) -> bool {
        self.meta.is_idx_vacant(idx)
    }

    #[inline]
    pub fn has_key(&self, key: Key) -> bool {
        self.meta.has_key(key)
    }

    #[inline]
    pub fn lookup_key(&self, idx: Idx) -> Option<Key> {
        self.meta.lookup_key(idx)
    }

    #[inline]
    pub fn lookup_idx(&self, key: Key) -> Option<Idx> {
        self.meta.lookup_idx(key)
    }

    #[inline]
    pub fn get_by_idx_mut(&mut self, idx: Idx) -> anyhow::Result<PageEntryMut<T>> {
        if self.is_idx_vacant(idx) {
            anyhow::bail!("idx is vacant");
        }

        Ok(unsafe {
            self.meta
                .nth_ptr_mut(self.data.as_mut_ptr(), idx.as_usize())
        })
    }

    #[inline]
    pub fn get_by_idx(&self, idx: Idx) -> anyhow::Result<PageEntryRef<T>> {
        if self.is_idx_vacant(idx) {
            anyhow::bail!("idx is vacant");
        }

        Ok(unsafe { self.meta.nth_ptr(self.data.as_ptr(), idx.as_usize()) })
    }

    #[inline]
    pub fn get_by_key_mut(&mut self, key: Key) -> anyhow::Result<PageEntryMut<T>> {
        let idx = if let Some(idx) = self.meta.lookup_idx(key) {
            idx
        } else {
            anyhow::bail!("key not found");
        };

        Ok(unsafe {
            self.meta
                .nth_ptr_mut(self.data.as_mut_ptr(), idx.as_usize())
        })
    }

    #[inline]
    pub fn get_by_key(&self, key: Key) -> anyhow::Result<PageEntryRef<T>> {
        let idx = if let Some(idx) = self.meta.lookup_idx(key) {
            idx
        } else {
            anyhow::bail!("key not found");
        };

        Ok(unsafe { self.meta.nth_ptr(self.data.as_ptr(), idx.as_usize()) })
    }

    #[inline]
    pub fn insert(&mut self, key: Key, val: T) -> anyhow::Result<Option<T>> {
        if let Some(idx) = self.lookup_idx(key) {
            let mut entry = self
                .get_by_idx_mut(idx)
                .expect("`idx` is known to be occupied");

            self.meta
                .replace_key(idx, key)
                .expect("`idx` is known to be occupied");

            Ok(Some(unsafe { entry.replace_val(val).assume_init() }))
        } else {
            let idx = self.meta.insert_key(key)?;

            let mut entry = self
                .get_by_idx_mut(idx)
                .expect("`idx` is known to be vacant");

            entry.replace_key(key);
            entry.replace_val(val);

            Ok(None)
        }
    }

    #[inline]
    pub fn delete(&mut self, key: Key) -> anyhow::Result<()> {
        self.meta.vacate(IdxOrKey::Key(key))?;

        Ok(())
    }

    #[inline]
    pub fn keys(&self) -> impl Iterator<Item = &Key> {
        self.meta.keys()
    }
}
