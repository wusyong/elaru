# elaru: A simple, fast, and memory safe least-recently-used (LRU) cache.

`elaru` avoids all unsafe operations while still achieves O(1) performance on `insert`, `get`,
and `remove_lru`. `fnv` feature is also provided for anyone looking for better performance on
small key size.


See the [documentation](https://docs.rs/elaru) for examples and more details.

Acknowledgement: This crate is heavily inspired by [uluru](https://crates.io/crates/uluru), a no_std lru implementation from servo.
