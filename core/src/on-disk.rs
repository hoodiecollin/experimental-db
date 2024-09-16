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

pub struct Page<T> {
    data: MmapMut,
    slots: Vec<Value<T>>,
    vacant: HashSet<usize>,
    length: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T> Page<T> {
    pub fn new() -> Self {
        let layout = Layout::array::<T>(1024).expect("Failed to create layout");
        let data = MmapMut::map_anon(layout.size()).expect("Failed to create mmap");

        #[cfg(debug_assertions)]
        {
            if !(data.as_ptr() as *const T).is_aligned() {
                panic!("Page is not aligned");
            }
        }

        let mut slots = Vec::<Value<T>>::with_capacity(1024);
        let mut vacant = HashSet::with_capacity(1024);
        for i in 0..1024 {
            let ptr = unsafe {
                let ptr = data.as_mut_ptr() as *mut MaybeUninit<T>;
                ptr.add(i)
            };
            slots.push(Value::new(ptr));
            vacant.insert(i);
        }

        Page {
            data,
            slots,
            vacant,
            length: 0,
            _marker: std::marker::PhantomData,
        }
    }
}
