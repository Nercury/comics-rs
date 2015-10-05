use std::collections::HashMap;
use std::path::{ Path, PathBuf };
use serde_json;
use std::io::prelude::*;
use std::fs::File;

use index_models::IndexRepr;

#[derive(Debug)]
pub struct IndexItem {
    title: String,
    slug: String,
    file: PathBuf,
    prev: Option<usize>,
    next: Option<usize>,
}

#[derive(Debug)]
pub struct Index {
    path: PathBuf,
    storage: Storage,
}

#[derive(Debug)]
struct Storage {
    items: Vec<IndexItem>,
    slug_map: HashMap<String, usize>,
}

#[derive(Debug)]
struct FoundIndex {
    pub title: String,
    pub file: PathBuf,
    pub prev_slug: Option<String>,
    pub next_slug: Option<String>,
}

impl Storage {
    fn from_file(path: &Path) -> Option<Storage> {
        let mut f = match File::open(path) {
            Ok(f) => f,
            _ => {
                println!("failed to open index storage");
                return None;
            },
        };

        let mut contents = String::new();
        if let Err(e) = f.read_to_string(&mut contents) {
            panic!("error reading index storage {:?}", e);
        }

        let deserialized: Vec<IndexRepr> = match serde_json::from_str(&contents) {
            Ok(contents) => contents,
            Err(e) => {
                panic!("error deserializing index storage {:?}", e);
            }
        };

        let mut storage = Storage::empty();

        for item in deserialized {
            storage.push(item.title, item.slug, &Path::new(&item.file));
        }

        println!("Storage {:#?}", storage);

        Some(storage)
    }

    fn empty() -> Storage {
        Storage {
            items: Vec::new(),
            slug_map: HashMap::new(),
        }
    }

    fn push<T: Into<String>, S: Into<String>>(&mut self, title: T, slug: S, file: &Path) {
        let title: String = title.into();
        let slug: String = slug.into();
        let prev_index = if self.items.len() == 0 {
            None
        } else {
            Some(self.items.len() - 1)
        };
        if let Some(last_index) = prev_index {
            if let Some(last_item) = self.items.get_mut(last_index) {
                last_item.next = Some(last_index + 1);
            }
        }
        self.items.push(IndexItem {
            title: title,
            slug: slug.clone(),
            file: file.into(),
            prev: prev_index,
            next: None,
        });
        self.slug_map.insert(slug, self.items.len() - 1);
    }
}

impl Index {
    pub fn from_file(path: &str) -> Index {
        let path = Path::new(path);
        let storage = match Storage::from_file(path) {
            Some(s) => s,
            None => Storage::empty(),
        };

        Index {
            path: path.into(),
            storage: storage,
        }
    }

    pub fn find<'r>(&self, slug: &'r str) -> Option<FoundIndex> {
        match self.storage.slug_map.get(slug) {
            Some(index) => {
                let item = &self.storage.items[*index];
                let next = match item.next {
                    Some(index) => Some(&self.storage.items[index]),
                    None => None,
                };
                let prev = match item.prev {
                    Some(index) => Some(&self.storage.items[index]),
                    None => None,
                };
                Some(FoundIndex {
                    title: item.title.clone(),
                    file: item.file.clone(),
                    prev_slug: match prev {
                        Some(pi) => Some(pi.slug.clone()),
                        None => None,
                    },
                    next_slug: match next {
                        Some(ni) => Some(ni.slug.clone()),
                        None => None,
                    },
                })
            },
            None => None,
        }
    }
}
