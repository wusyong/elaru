//! A simple, fast, and memory safe least-recently-used (LRU) cache.
//!
//! `elaru` avoids all unsafe operations while still achieves O(1) performance on `insert`, `get`,
//! and `remove_lru`. `fnv` feature is also provided for anyone looking for better performance on
//! small key size.
//!
//! See the [`LRUCache`] docs for more details.

#![warn(missing_debug_implementations, missing_docs, unreachable_pub)]

#[cfg(feature = "fnv")]
use fnv::FnvBuildHasher;
use std::collections::{hash_map::Entry as MapEntry, HashMap};

/// A LRU cache builds on top of the HashMap from standard library.
///
/// `LRUCache` uses `std::collections::HashMap` for storage. It provides `O(1)` performance on
/// `insert`, `get`, `remove_lru` and many other APIs.
///
/// All entries are linked inlined within the `LRUCache` without raw pointer manipulation, so it is
/// complete memory safe and doesn't suffer any undefined behavior. A linked list is used to record
/// the cache order, so the items themselves do not need to be moved when the order changes.
/// (This is important for speed if the items are large.)
///
/// # Example
///
/// ```
/// use elaru::{LRUCache, Entry};
///
/// // Create an empty cache, then insert some items.
/// let mut cache = LRUCache::new(3);
/// cache.insert(1, "Mercury");
/// cache.insert(2, "Venus");
/// cache.insert(3, "Earth");
///
/// // Use the `get` method to retrieve the value from the cache with given key.
/// // This also "touches" the entry, marking it most-recently-used.
/// let item = cache.get(&1).unwrap();
/// assert_eq!(item, &"Mercury");
///
/// // If the cache is full, inserting a new item evicts the least-recently-used item:
/// cache.insert(4, "Mars");
/// assert!(cache.get(&2).is_none());
/// ```
#[derive(Debug, Clone)]
pub struct LRUCache<T> {
    /// The most-recently-used entry is at index `head`. The entries form a linked list, linked to
    /// each other by key within the `entries` map.
    #[cfg(not(feature = "fnv"))]
    entries: HashMap<u16, Entry<T>>,
    #[cfg(feature = "fnv")]
    entries: HashMap<u16, Entry<T>, FnvBuildHasher>,
    /// Index of the first entry. If the cache is empty, ignore this field.
    head: u16,
    /// Index of the last entry. If the cache is empty, ignore this field.
    tail: u16,
    capacity: usize,
}

/// An entry in an LRUCache.
#[derive(Debug, Clone)]
pub struct Entry<T> {
    val: T,
    /// Index of the previous entry. If this entry is the head, ignore this field.
    prev: u16,
    /// Index of the next entry. If this entry is the tail, ignore this field.
    next: u16,
}

impl<T> LRUCache<T> {
    /// Create a new LRU cache that can hold `capacity` of entries.
    pub fn new(capacity: usize) -> Self {
        let cache = LRUCache {
            entries: HashMap::default(),
            head: 0,
            tail: 0,
            capacity,
        };
        assert!(
            cache.capacity < u16::max_value() as usize,
            "Capacity overflow"
        );
        cache
    }

    /// Returns the number of elements in the cache.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns the capacity of the cache.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the entry in the list with given key.
    pub fn get(&mut self, key: &u16) -> Option<&T> {
        if self.entries.contains_key(key) {
            self.touch_index(*key);
        }
        self.entries.get(key).map(|e| &e.val)
    }

    /// Returns a mutable reference to the entry in the list with given key.
    pub fn get_mut(&mut self, key: &u16) -> Option<&mut T> {
        if self.entries.contains_key(key) {
            self.touch_index(*key);
        }
        self.entries.get_mut(key).map(|e| &mut e.val)
    }

    /// Insert a given key in the cache. Return old value if the key is present.
    ///
    /// This item becomes the front (most-recently-used) item in the cache.  If the cache is full,
    /// the back (least-recently-used) item will be removed.
    pub fn insert(&mut self, key: u16, val: T) -> Option<T> {
        // If the cache is full, remove the tail entry.
        if self.entries.len() == self.capacity {
            #[cfg(not(feature = "unbound"))]
            self.remove_lru().expect("Invalid entry access");
        }

        let old = match self.entries.entry(key) {
            MapEntry::Occupied(mut e) => {
                let old_val = e.insert(Entry {
                    val,
                    prev: e.get().prev,
                    next: e.get().next,
                });
                Some(old_val.val)
            }
            MapEntry::Vacant(e) => {
                e.insert(Entry {
                    val,
                    prev: 0,
                    next: 0,
                });
                None
            }
        };

        self.push_front(key);
        old
    }

    /// Remove an entry from the linked list.
    pub fn remove_lru(&mut self) -> Option<(u16, T)> {
        self.entries.remove(&self.tail).map(|old_tail| {
            let old_key = self.tail;
            let new_tail = old_tail.prev;
            self.tail = new_tail;
            (old_key, old_tail.val)
        })
    }

    /// Clear all elements from the cache.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Iterate over the contents of this cache.
    pub fn iter(&self) -> Iter<T> {
        Iter {
            pos: self.head,
            done: self.entries.len() == 0,
            cache: self,
        }
    }

    /// Touch a given entry, putting it first in the list.
    #[inline]
    fn touch_index(&mut self, idx: u16) {
        if idx != self.head {
            self.evict(idx);
            self.push_front(idx);
        }
    }

    /// Evict an entry from the linked list.
    /// Note this doesn't remove the entry from the cache.
    fn evict(&mut self, i: u16) {
        let evicted = self.entries.get(&i).expect("Invalid entry access");
        let prev = evicted.prev;
        let next = evicted.next;

        if i == self.head {
            self.head = next;
        } else {
            self.entries
                .get_mut(&prev)
                .expect("Invalid entry access")
                .next = next;
        }

        if i == self.tail {
            self.tail = prev;
        } else {
            self.entries
                .get_mut(&next)
                .expect("Invalid entry access")
                .prev = prev;
        }
    }

    /// Insert a new entry at the head of the list.
    fn push_front(&mut self, i: u16) {
        if self.entries.len() == 1 {
            self.tail = i;
        } else {
            self.entries.get_mut(&i).expect("Invalid entry access").next = self.head;
            self.entries
                .get_mut(&self.head)
                .expect("Invalid entry access")
                .prev = i;
        }
        self.head = i;
    }
}

/// Mutable iterator over values in an LRUCache, from most-recently-used to least-recently-used.
#[derive(Debug)]
pub struct Iter<'a, T> {
    cache: &'a LRUCache<T>,
    pos: u16,
    done: bool,
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: 'a,
{
    type Item = (u16, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        // Use a raw pointer because the compiler doesn't know that subsequent calls can't alias.
        //let entry = unsafe { &mut *(&mut self.cache.entries[self.pos as usize] as *mut Entry<T>) };
        let (key, entry) = self
            .cache
            .entries
            .get_key_value(&self.pos)
            .expect("Invalid entry access");

        if self.pos == self.cache.tail {
            self.done = true;
        }
        self.pos = entry.next;

        Some((*key, &entry.val))
    }
}
