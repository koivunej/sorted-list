#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(unused_imports)]

//! Simple sorted list collection like the one found in the .NET collections library.

use std::fmt;

/// `SortedList` stores multiple `(K, V)` tuples ordered by K, then in the order of insertion for `V`.
/// Implmented using two `Vec` this should be fast for in-order inserts and quite bad in the
/// worst-case of reverse insertion order.
///
/// # Example
///
/// ```
/// use sorted_list::SortedList;
///
/// let mut list: SortedList<u32, u8> = SortedList::new();
/// list.insert(0, 0);
/// list.insert(1, 1);
/// list.insert(0, 2);
///
/// assert_eq!(
///     list.iter().collect::<Vec<_>>(),
///     vec![(&0, &0), (&0, &2), (&1, &1)]);
/// ```
pub struct SortedList<K: Ord, V: PartialEq> {
    keys: Vec<K>,
    values: Vec<V>,
}

impl<K: Ord, V: PartialEq> SortedList<K, V> {
    /// Creates a new as small as possible `SortedList`
    pub fn new() -> Self {
        SortedList { keys: Vec::new(), values: Vec::new() }
    }

    /// Returns the number of tuples
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Returns `true` if the `(key, value)` did not exist in the sorted list before and it exists now,
    /// `false` otherwise.
    pub fn insert(&mut self, key: K, value: V) -> bool {
        match self.keys.binary_search(&key) {
            Ok(found_at) => {
                let insertion_position = self.find_insertion_positition(found_at, &key, &value);

                if let Some(insertion_position) = insertion_position {
                    insertion_position.insert(key, value, &mut self.keys, &mut self.values);
                    true
                } else {
                    false
                }
            },
            Err(insert_at) => {
                self.keys.insert(insert_at, key);
                self.values.insert(insert_at, value);

                true
            }
        }
    }

    /// Returns an iterator over the values of a specific key
    pub fn values_of<'a>(&'a self, key: &'a K) -> ValuesOf<'a, K, V> {
        let first = self.keys.iter().position(|existing| key == existing);

        ValuesOf::new(first, key, self.iter())
    }

    fn find_insertion_positition(&self, from: usize, key: &K, value: &V) -> Option<InsertionPosition> {
        let mut keys = self.keys.iter().skip(from);
        let mut values = self.values.iter().skip(from);

        let mut index: usize = from;

        loop {
            index += 1;

            match (keys.next(), values.next()) {
                (Some(other_key), Some(other_value)) => {
                    if key == other_key {
                        if value == other_value {
                            // found it already
                            return None;
                        }
                    } else {
                        // we ran past the matching keys, insert before
                        return Some(InsertionPosition::Before(index));
                    }
                },
                (None, None) => {
                    return Some(InsertionPosition::Last);
                }
                (_, _) => unreachable!(),
            };
        }
    }

    /// Iterate all stored tuples, keys in order, values in insertion order
    pub fn iter(&self) -> Tuples<K, V> {
        Tuples {
            keys: self.keys.iter(),
            values: self.values.iter(),
        }
    }

    /// Iterate over all keys, can contain duplicates
    pub fn keys(&self) -> ::std::slice::Iter<K> {
        self.keys.iter()
    }

    /// Iterate over all values
    pub fn values(&self) -> ::std::slice::Iter<V> {
        self.values.iter()
    }
}

impl<K: Ord + fmt::Debug, V: PartialEq + fmt::Debug> fmt::Debug for SortedList<K, V> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "SortedList {{ {:?} }}", &self.iter())
    }
}

/// Helper value for knowning where to insert the value
enum InsertionPosition {
    Before(usize),
    Last
}

impl InsertionPosition {
    fn insert<K, V>(self, key: K, value: V, keys: &mut Vec<K>, values: &mut Vec<V>) {
        match self {
            InsertionPosition::Before(index) => {
                keys.insert(index - 1, key);
                values.insert(index - 1, value);

                assert_eq!(keys.len(), values.len());
            },
            InsertionPosition::Last => {
                keys.push(key);
                values.push(value);

                assert_eq!(keys.len(), values.len());
            }
        }
    }
}

/// Iterator over tuples stored in `SortedList`
pub struct Tuples<'a, K: 'a, V: 'a> {
    keys: ::std::slice::Iter<'a, K>,
    values: ::std::slice::Iter<'a, V>,
}

impl<'a, K, V> Iterator for Tuples<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.keys.next(), self.values.next()) {
            (Some(k), Some(v)) => Some((k, v)),
            (None, None) => None,
            _ => unreachable!(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.keys.size_hint()
    }
}

impl<'a, K, V> Clone for Tuples<'a, K, V> {
    fn clone(&self) -> Self {
        Tuples {
            keys: self.keys.clone(),
            values: self.values.clone(),
        }
    }
}

impl<'a, K: Ord + fmt::Debug, V: PartialEq + fmt::Debug> fmt::Debug for Tuples<'a, K, V> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let remaining = self.size_hint().0;
        let mut clone = self.clone();
        let mut idx = 0;
        write!(fmt, "[")?;
        while let Some(tuple) = clone.next() {
            if idx == remaining - 1 {
                write!(fmt, "{:?}", tuple)?;
            } else {
                write!(fmt, "{:?}, ", tuple)?;
            }
            idx += 1;
        }
        write!(fmt, "]")
    }
}

impl<'a, K, V> ExactSizeIterator for Tuples<'a, K, V> {}

/// Iterator over values of a specific key stored in `SortedList`
pub struct ValuesOf<'a, K: 'a + PartialEq, V: 'a> {
    iter: Option<::std::iter::Skip<Tuples<'a, K, V>>>,
    key: &'a K,
}

impl<'a, K: PartialEq, V> Clone for ValuesOf<'a, K, V> {
    fn clone(&self) -> Self {
        ValuesOf {
            iter: self.iter.clone(),
            key: self.key.clone(),
        }
    }
}

impl<'a, K: PartialEq, V> ValuesOf<'a, K, V> {
    fn new(first_index: Option<usize>, key: &'a K, iter: Tuples<'a, K, V>) -> Self {
        let iter = match first_index {
            Some(first_index) => {
                Some(iter.skip(first_index))
            },
            None => None,
        };
        ValuesOf { iter, key }
    }
}

impl<'a, K: PartialEq, V> Iterator for ValuesOf<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.as_mut() {
            Some(mut iter) => {
                match iter.next() {
                    Some((k, v)) => if self.key == k {
                        Some(v)
                    } else {
                        None
                    },
                    None => None
                }
            },
            None => None
        }
    }
}

impl<'a, K: PartialEq, V: fmt::Debug> fmt::Debug for ValuesOf<'a, K, V> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut clone = self.clone().peekable();
        write!(fmt, "[")?;
        while let Some(val) = clone.next() {
            if clone.peek().is_some() {
                write!(fmt, "{:?}, ", val)?;
            } else {
                write!(fmt, "{:?}", val)?;
            }
        }
        write!(fmt, "]")
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use super::SortedList;

    /// Extension trait with asserting methods
    trait SortedListExt<K, V> {
        fn insert_only_new(&mut self, key: K, value: V);
    }

    impl<K: Debug + Clone + Ord, V: Debug + Clone + PartialEq> SortedListExt<K, V> for SortedList<K, V> {
        fn insert_only_new(&mut self, key: K, value: V) {
            let cloned_key = key.clone();
            let cloned_value = value.clone();

            assert!(self.insert(key, value), "pair existed already: ({:?}, {:?})", cloned_key, cloned_value);
        }
    }

    #[test]
    fn insert_in_order_and_iterate() {
        let mut list = SortedList::new();
        list.insert_only_new(0u32, 0u8);
        list.insert_only_new(1u32, 4u8);

        let mut iter = list.iter();

        assert_eq!(iter.next(), Some((&0, &0)));
        assert_eq!(iter.next(), Some((&1, &4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn insert_out_of_order_and_iterate() {
        let mut list = SortedList::new();
        list.insert_only_new(1u32, 4u8);
        list.insert_only_new(0u32, 0u8);

        let mut iter = list.iter();

        assert_eq!(iter.next(), Some((&0, &0)));
        assert_eq!(iter.next(), Some((&1, &4)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn insert_duplicate() {
        let mut list = SortedList::new();
        assert!(list.insert(1u32, 4u8));
        assert!(!list.insert(1u32, 4u8));
    }

    #[test]
    fn insert_multiple_in_order() {
        let mut list = SortedList::new();
        list.insert_only_new(0u32, 0u8);
        list.insert_only_new(0u32, 1u8);
        list.insert_only_new(0u32, 2u8);
        list.insert_only_new(0u32, 3u8);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some((&0, &0)));
        assert_eq!(iter.next(), Some((&0, &1)));
        assert_eq!(iter.next(), Some((&0, &2)));
        assert_eq!(iter.next(), Some((&0, &3)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn multiple_values_are_iterated_in_insertion_order() {
        let mut list = SortedList::new();
        list.insert_only_new(0u32, 3u8);
        list.insert_only_new(0u32, 2u8);
        list.insert_only_new(0u32, 1u8);
        list.insert_only_new(0u32, 0u8);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some((&0, &3)));
        assert_eq!(iter.next(), Some((&0, &2)));
        assert_eq!(iter.next(), Some((&0, &1)));
        assert_eq!(iter.next(), Some((&0, &0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iterate_over_mixed_in_order() {
        let mut list = SortedList::new();
        list.insert_only_new(0u32, 0u8);
        list.insert_only_new(0u32, 1u8);
        list.insert_only_new(0u32, 2u8);
        list.insert_only_new(0u32, 3u8);
        list.insert_only_new(1u32, 4u8);
        list.insert_only_new(2u32, 5u8);
        list.insert_only_new(2u32, 6u8);
        list.insert_only_new(3u32, 7u8);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some((&0, &0)));
        assert_eq!(iter.next(), Some((&0, &1)));
        assert_eq!(iter.next(), Some((&0, &2)));
        assert_eq!(iter.next(), Some((&0, &3)));
        assert_eq!(iter.next(), Some((&1, &4)));
        assert_eq!(iter.next(), Some((&2, &5)));
        assert_eq!(iter.next(), Some((&2, &6)));
        assert_eq!(iter.next(), Some((&3, &7)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iterate_over_mixed_out_of_order() {
        let mut list = SortedList::new();
        list.insert_only_new(3u32, 7u8);
        list.insert_only_new(0u32, 0u8);
        list.insert_only_new(1u32, 4u8);
        list.insert_only_new(0u32, 1u8);

        println!("{:?}", list);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some((&0, &0)));
        assert_eq!(iter.next(), Some((&0, &1)));
        assert_eq!(iter.next(), Some((&1, &4)));
        assert_eq!(iter.next(), Some((&3, &7)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iterate_values_of() {
        let mut list = SortedList::new();
        list.insert_only_new(1u32, 4u8);
        list.insert_only_new(0u32, 0u8);
        list.insert_only_new(0u32, 1u8);
        list.insert_only_new(2u32, 5u8);
        list.insert_only_new(0u32, 2u8);
        list.insert_only_new(3u32, 7u8);
        list.insert_only_new(0u32, 3u8);
        list.insert_only_new(2u32, 6u8);

        let q = 0;
        let mut values_of = list.values_of(&q);
        assert_eq!(values_of.next(), Some(&0));
        assert_eq!(values_of.next(), Some(&1));
        assert_eq!(values_of.next(), Some(&2));
        assert_eq!(values_of.next(), Some(&3));
        assert_eq!(values_of.next(), None);

        let q = 1;
        let mut values_of = list.values_of(&q);
        assert_eq!(values_of.next(), Some(&4));
        assert_eq!(values_of.next(), None);

        let q = 2;
        let mut values_of = list.values_of(&q);
        assert_eq!(values_of.next(), Some(&5));
        assert_eq!(values_of.next(), Some(&6));
        assert_eq!(values_of.next(), None);

        let q = 3;
        let mut values_of = list.values_of(&q);
        assert_eq!(values_of.next(), Some(&7));
        assert_eq!(values_of.next(), None);
    }
}
