# SortedList

[![Build Status](https://travis-ci.org/koivunej/sorted-list.svg?branch=master)](https://travis-ci.org/koivunej/sorted-list)
[![Docs](https://docs.rs/sorted-list/badge.svg)](https://docs.rs/crate/sorted-list)
[![crates.io](https://img.shields.io/crates/v/sorted-list.svg)](https://crates.io/crates/sorted-list)

Data structure similar to [SortedList found in .NET](https://msdn.microsoft.com/en-us/library/ms132319(v=vs.110).aspx) for rust.
Naive implementation based on `Vec<K>` and `Vec<V>`.
Same key can be mapped to multiple values, and the values are stored in insertion order.

Unsupported:

 * deletion
 * ~range queries~ (requires feature = "nightly" and a nightly compiler)

## Cargo.toml

```
[dependencies]
sorted-list = "0.2"
```

or:

```
[dependencies]
sorted-list = { git = "https://github.com/koivunej/sorted-list.git" }
```

## License

MIT.
