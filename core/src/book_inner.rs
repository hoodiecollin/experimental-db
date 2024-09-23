use std::{
    collections::{BTreeSet, HashMap},
    fs,
};

use crate::{page::Page, page_layout::PAGE_SIZE, BookId, Idx, Key, DATA_DIR};

#[derive(Debug)]
pub struct BookInner<T> {
    id: BookId,
    pages: Vec<Page<T>>,
    key_lookup: HashMap<Key, Idx>,
    partial: BTreeSet<Idx>,
}

impl<T> BookInner<T> {
    pub fn new(id: BookId) -> anyhow::Result<Self> {
        let pages_dir = DATA_DIR.join(format!("books/{}/pages", id.val));

        fs::create_dir_all(&pages_dir)
            .map_err(|e| anyhow::anyhow!(e).context(format!("failed to ensure {:?}", pages_dir)))?;

        let mut page_files = fs::read_dir(&pages_dir)
            .map_err(|e| anyhow::anyhow!(e).context(format!("failed to read {:?}", pages_dir)))?
            .map(|entry| entry.map_err(|e| anyhow::anyhow!(e).context("failed to read entry")))
            .collect::<anyhow::Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
            .map(|entry| {
                let path = entry.path();
                let file_name = path.file_name().unwrap().to_str().unwrap();
                let idx = file_name.parse::<u32>().unwrap();
                (idx, path)
            })
            .collect::<Vec<_>>();

        if page_files.is_empty() {
            page_files.push((0, pages_dir.join("0")));
        }

        page_files.sort_unstable_by_key(|(idx, _)| *idx);

        let mut key_lookup = HashMap::with_capacity(PAGE_SIZE * page_files.len());
        let mut partial = BTreeSet::new();

        let pages = page_files
            .into_iter()
            .map(|(i, path)| -> anyhow::Result<Option<Page<T>>> {
                let file = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&path)
                    .map_err(|e| {
                        anyhow::anyhow!(e).context(format!("failed to open {:?}", path))
                    })?;

                let page = if file.metadata()?.len() == 0 {
                    Page::new(&file)?
                } else {
                    Page::parse(&file)?
                };

                let page_idx = Idx::new(i);
                let page_guard = page.read();

                if !page_guard.is_full() {
                    partial.insert(page_idx);
                }

                key_lookup.extend(page_guard.keys().map(|key| (*key, page_idx)));

                Ok(Some(page))
            })
            .collect::<anyhow::Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();

        Ok(BookInner {
            id,
            pages,
            key_lookup,
            partial,
        })
    }

    pub fn len(&self) -> usize {
        self.key_lookup.len()
    }

    pub fn has_key(&self, key: Key) -> bool {
        self.key_lookup.contains_key(&key)
    }

    pub fn insert(&mut self, key: Key, val: T) -> anyhow::Result<Option<T>> {
        if self.has_key(key) {
            anyhow::bail!("key already exists");
        }

        let page_idx = if let Some(page_idx) = self.partial.iter().next() {
            *page_idx
        } else {
            let page_idx = Idx::new(self.pages.len() as u32);
            let path = DATA_DIR.join(format!("books/{}/pages/{}", self.id.val, page_idx.val));

            let file = match fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&path)
            {
                Ok(file) => file,
                Err(e) => {
                    return Err(anyhow::anyhow!(e).context(format!("failed to open {:?}", path)))
                }
            };

            let page = match Page::new(&file) {
                Ok(page) => page,
                Err(e) => {
                    return Err(
                        anyhow::anyhow!(e).context(format!("failed to create page {:?}", path))
                    )
                }
            };

            self.pages.push(page);
            self.partial.insert(page_idx);
            page_idx
        };

        self.key_lookup.insert(key, page_idx);

        let page = &self.pages[page_idx.as_usize()];
        let ret = { page.write().insert(key, val)? };

        if page.read().is_full() {
            self.partial.remove(&page_idx);
        }

        Ok(ret)
    }

    pub fn delete(&mut self, key: Key) -> anyhow::Result<()> {
        let page_idx = if let Some(page_idx) = self.key_lookup.get(&key) {
            *page_idx
        } else {
            anyhow::bail!("key not found")
        };

        let mut page_guard = self.pages[page_idx.as_usize()].read();

        if page_guard.is_full() {
            self.partial.insert(page_idx);
        }

        page_guard.with_upgraded(|page_guard| page_guard.delete(key))?;

        Ok(())
    }
}
