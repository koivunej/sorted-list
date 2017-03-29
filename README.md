# SortedList

Data structure similar to [SortedList found in .NET](https://msdn.microsoft.com/en-us/library/ms132319(v=vs.110).aspx) for rust.
Naive implementation based on two `Vec<K>` and `Vec<V>`.
Same key can be mapped to multiple values, and the values are stored in insertion order.

Unsupported:

 * deletion
 * ~range queries~ (requires feature = "nightly" and a nightly compiler)

## Cargo.toml

```
[dependencies]
sorted_list = "0.1"
```

## Documentation

Probably soon at docs.rs.

## License

MIT.
