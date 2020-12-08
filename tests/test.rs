use elaru::*;

/// Convenience function for test assertions
fn items<T>(cache: &LRUCache<T>) -> Vec<u16>
where
    T: Clone,
{
    cache.iter().map(|(x, _)| x.clone()).collect()
}

#[test]
fn empty() {
    let mut cache: LRUCache<u8> = LRUCache::new(4);
    assert_eq!(cache.len(), 0);
    assert_eq!(items(&mut cache), []);
}

#[test]
fn insert() {
    let mut cache = LRUCache::new(4);
    cache.insert(1, "a");
    assert_eq!(cache.len(), 1);
    cache.insert(2, "b");
    assert_eq!(cache.len(), 2);
    cache.insert(3, "c");
    assert_eq!(cache.len(), 3);
    cache.insert(4, "d");
    assert_eq!(cache.len(), 4);
    assert_eq!(
        items(&cache),
        [4, 3, 2, 1],
        "Ordered from most- to least-recent."
    );
    cache.insert(5, "e");
    dbg!(&cache);
    assert_eq!(cache.len(), 4);
    assert_eq!(
        items(&mut cache),
        [5, 4, 3, 2],
        "Least-recently-used item cleared."
    );

    cache.insert(6, "f");
    cache.insert(7, "g");
    cache.insert(8, "h");
    cache.insert(9, "i");
    assert_eq!(
        items(&mut cache),
        [9, 8, 7, 6],
        "Least-recently-used item cleared."
    );
}

#[test]
fn lookup() {
    let mut cache = LRUCache::new(4);
    cache.insert(1, 100);
    cache.insert(2, 200);
    cache.insert(3, 300);
    cache.insert(4, 400);

    let result = cache.get(&5);
    assert_eq!(result, None, "Cache miss.");
    assert_eq!(items(&mut cache), [4, 3, 2, 1], "Order not changed.");

    // Cache hit
    let result = cache.get_mut(&3);
    assert_eq!(result, Some(&mut 300), "Cache hit.");
    assert_eq!(
        items(&mut cache),
        [3, 4, 2, 1],
        "Matching item moved to front."
    );
}

#[test]
fn clear() {
    let mut cache = LRUCache::new(4);
    cache.insert(1, 100);
    cache.clear();
    assert_eq!(items(&mut cache), [], "all items cleared");

    cache.insert(1, 100);
    cache.insert(2, 200);
    cache.insert(3, 300);
    cache.insert(4, 400);
    assert_eq!(items(&mut cache), [4, 3, 2, 1]);
    cache.clear();
    assert_eq!(items(&mut cache), [], "all items cleared again");
}

#[test]
fn remove_lru() {
    let mut cache = LRUCache::new(4);

    cache.insert(1, 100);
    cache.insert(2, 200);
    cache.insert(3, 300);
    cache.insert(4, 400);
    cache.remove_lru();
    assert_eq!(
        items(&mut cache),
        [4, 3, 2],
        "Least-recently-used item cleared."
    );
}

#[test]
fn iter() {
    let mut cache = LRUCache::new(4);

    cache.insert(1, 100);
    cache.insert(2, 200);
    cache.insert(3, 300);
    cache.insert(4, 400);

    let mut iter = cache.iter();
    assert_eq!(iter.next(), Some((4, &400)));
    assert_eq!(iter.next(), Some((3, &300)));
    assert_eq!(iter.next(), Some((2, &200)));
    assert_eq!(iter.next(), Some((1, &100)));
    assert_eq!(iter.next(), None);
}
