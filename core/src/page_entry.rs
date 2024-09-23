use std::{mem::MaybeUninit, ptr};

use crate::Key;

#[repr(C, packed)]
#[derive(Debug)]
pub struct PageEntry<T> {
    key: Key,
    val: MaybeUninit<T>,
}

impl<T> PageEntry<T> {
    #[inline]
    pub fn new(key: Key, val: T) -> Self {
        Self {
            key,
            val: MaybeUninit::new(val),
        }
    }

    #[inline]
    pub fn as_ref(ptr: *const Self) -> PageEntryRef<T> {
        PageEntryRef { ptr }
    }

    #[inline]
    pub fn as_mut(ptr: *mut Self) -> PageEntryMut<T> {
        PageEntryMut { ptr }
    }
}

impl<T> Default for PageEntry<T> {
    #[inline]
    fn default() -> Self {
        Self {
            key: Key::default(),
            val: MaybeUninit::uninit(),
        }
    }
}

pub struct PageEntryRef<T> {
    pub(self) ptr: *const PageEntry<T>,
}

impl<T> PageEntryRef<T> {
    #[inline]
    pub unsafe fn add(&self, offset: usize) -> Self {
        Self {
            ptr: self.ptr.add(offset),
        }
    }

    #[inline]
    pub fn key(&self) -> Key {
        unsafe {
            let entry = &*self.ptr;
            ptr::addr_of!(entry.key).read_unaligned()
        }
    }

    #[inline]
    pub fn val(&mut self) -> MaybeUninit<T> {
        unsafe {
            let entry = &*self.ptr;
            ptr::addr_of!(entry.val).read_unaligned()
        }
    }
}

pub struct PageEntryMut<T> {
    pub(self) ptr: *mut PageEntry<T>,
}

impl<T> PageEntryMut<T> {
    #[inline]
    pub unsafe fn add(&self, offset: usize) -> Self {
        Self {
            ptr: self.ptr.add(offset),
        }
    }

    #[inline]
    pub fn key(&mut self) -> Key {
        unsafe {
            let entry = &*self.ptr;
            ptr::addr_of!(entry.key).read_unaligned()
        }
    }

    #[inline]
    fn set_key(&mut self, key: Key) {
        unsafe {
            let entry = &mut *self.ptr;
            ptr::addr_of_mut!(entry.key).write_unaligned(key);
        }
    }

    #[inline]
    pub fn replace_key(&mut self, key: Key) -> Key {
        let old = self.key();
        self.set_key(key);
        old
    }

    #[inline]
    pub fn val(&mut self) -> MaybeUninit<T> {
        unsafe {
            let entry = &*self.ptr;
            ptr::addr_of!(entry.val).read_unaligned()
        }
    }

    #[inline]
    fn set_val(&mut self, val: T) {
        unsafe {
            let entry = &mut *self.ptr;
            ptr::addr_of_mut!(entry.val).write_unaligned(MaybeUninit::new(val));
        }
    }

    #[inline]
    pub fn replace_val(&mut self, val: T) -> MaybeUninit<T> {
        let old = self.val();
        self.set_val(val);
        old
    }
}
