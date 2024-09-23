use std::{
    alloc::Layout,
    mem::{align_of, size_of},
};

use crate::{
    page_entry::{PageEntry, PageEntryMut, PageEntryRef},
    Idx, Key,
};

// 1MB page size
pub const PAGE_SIZE: usize = 128; //1024 * 1024;

#[derive(Debug, PartialEq)]
pub struct PageLayout<T> {
    pub cap: usize,
    pub elem_layout: Layout,
    pub total_usage: usize,
    pub bitmap_bytes: usize,
    pub wasted_bytes: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Clone for PageLayout<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for PageLayout<T> {}

impl<T> PageLayout<T> {
    pub fn new() -> Self {
        let total_memory: usize = PAGE_SIZE;
        let align = align_of::<PageEntry<T>>();
        let size = size_of::<PageEntry<T>>();

        // We need to find the maximum number of elements `cap` that fit in memory
        let mut cap = (total_memory - 1) / size; // start with an upper bound guess

        loop {
            // Calculate the number of bytes required for the bitmap
            let bitmap_bytes = (cap + 7) / 8;

            // Calculate the first aligned address after the bitmap
            let array_start = (bitmap_bytes + align - 1) & !(align - 1); // aligned offset

            // Total memory usage: bitmap + array (cap elements)
            let total_usage = array_start + cap * size;

            if total_usage <= total_memory {
                // We've found the valid `cap` that fits
                return PageLayout {
                    cap,
                    elem_layout: Layout::from_size_align(size, align).expect("Invalid layout"),
                    total_usage,
                    bitmap_bytes,
                    wasted_bytes: total_memory - total_usage,
                    _marker: std::marker::PhantomData,
                };
            }

            // Otherwise, reduce `cap` and try again
            cap -= 1;
        }
    }

    #[inline]
    pub unsafe fn array_ptr_mut(&self, data_ptr: *mut u8) -> PageEntryMut<T> {
        let bitmap_bytes = (self.cap + 7) / 8;
        let array_start =
            (bitmap_bytes + self.elem_layout.align() - 1) & !(self.elem_layout.align() - 1);

        PageEntry::as_mut(data_ptr.add(array_start) as *mut _)
    }

    #[inline]
    pub unsafe fn array_ptr(&self, data_ptr: *const u8) -> PageEntryRef<T> {
        let bitmap_bytes = (self.cap + 7) / 8;
        let array_start =
            (bitmap_bytes + self.elem_layout.align() - 1) & !(self.elem_layout.align() - 1);

        PageEntry::as_ref(data_ptr.add(array_start) as *const _)
    }

    #[inline]
    pub unsafe fn nth_ptr_mut(&self, data_ptr: *mut u8, n: usize) -> PageEntryMut<T> {
        self.array_ptr_mut(data_ptr)
            .add(n * self.elem_layout.size())
    }

    #[inline]
    pub unsafe fn nth_ptr(&self, data_ptr: *const u8, n: usize) -> PageEntryRef<T> {
        self.array_ptr(data_ptr).add(n * self.elem_layout.size())
    }

    #[inline]
    pub unsafe fn nth_is_vacant(&self, data_ptr: *const u8, n: usize) -> bool {
        let byte = n / 8;
        let bit = n % 8;
        let mask = 1 << bit;
        if *data_ptr.add(byte) & mask != 0 {
            false
        } else {
            true
        }
    }

    #[inline]
    pub unsafe fn page_entry_iter<'a, 'b>(
        &'a self,
        file_content: &'b [u8],
    ) -> anyhow::Result<PageEntryIter<'b, T>> {
        PageEntryIter::new(file_content, *self)
    }
}

pub struct PageEntryIter<'a, T> {
    data: &'a [u8],
    step: usize,
    layout: PageLayout<T>,
    error: Option<anyhow::Error>,
}

impl<'a, T> PageEntryIter<'a, T> {
    pub fn new(data: &'a [u8], layout: PageLayout<T>) -> anyhow::Result<Self> {
        Ok(Self {
            data,
            step: 0,
            layout,
            error: None,
        })
    }
}

impl<'a, T> Iterator for PageEntryIter<'a, T> {
    type Item = (Idx, Option<Key>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.error.is_some() {
            return None;
        }

        let len = self.layout.cap;
        let data_ptr = self.data.as_ptr();

        while self.step < len {
            let step = self.step;
            self.step += 1;

            let idx = Idx::new((step) as u32);
            let vacant = unsafe { self.layout.nth_is_vacant(data_ptr, step) };

            if vacant {
                return Some((idx, None));
            } else {
                return Some((
                    idx,
                    Some(unsafe { self.layout.nth_ptr(data_ptr, step).key() }),
                ));
            }
        }

        None
    }
}
