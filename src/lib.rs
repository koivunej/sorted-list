#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![allow(unused_imports)]

#![cfg_attr(feature = "nightly", feature(collections_bound))]
#![cfg_attr(feature = "nightly", feature(collections_range))]

//! Simple sorted list collection like the one found in the .NET collections library.

use std::fmt;

#[cfg(feature = "nightly")]
use std::borrow::Borrow;

#[cfg(feature = "nightly")]
use std::collections::Bound::*;

#[cfg(feature = "nightly")]
use std::collections::range::RangeArgument;

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
        /// FIXME: binary search here
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
            keys: &self.keys,
            values: &self.values,
            index: 0,
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

#[cfg(feature = "nightly")]
impl<K: Ord + PartialEq, V: PartialEq> SortedList<K, V> {
    /// Returns an iterator over the specified range of tuples
    pub fn range<R>(&self, range: R) -> Range<K, V> where R: RangeArgument<K>, {

        fn look_left_from<K: Ord>(key: &K, vec: &Vec<K>, mut pos: usize, offset: usize) -> Option<usize> {
            while pos > 0 && key == &vec[pos] { pos -= 1; }
            if pos == 0 {
                None
            } else {
                Some(pos + offset)
            }
        }

        fn look_right_from<K: Ord>(key: &K, vec: &Vec<K>, mut pos: usize) -> Option<usize> {
            while pos < vec.len() && key == &vec[pos] { pos += 1; }
            if pos == vec.len() {
                None
            } else {
                Some(pos)
            }
        }

        let start = match range.start() {
            Included(key) => {
                match self.keys.binary_search(key) {
                    Ok(pos) => Some(look_left_from(key, &self.keys, pos, 1).unwrap_or(0)),
                    Err(pos) => look_left_from(key, &self.keys, pos, 0),
                }
            },
            Excluded(key) => {
                let pos = match self.keys.binary_search(key) {
                    Ok(pos) => pos,
                    Err(pos) => pos,
                };

                look_right_from(key, &self.keys, pos)
            },
            Unbounded => {
                Some(0)
            }
        };

        let end = match range.end() {
            Included(key) => {
                let pos = match self.keys.binary_search(key) {
                    Ok(pos) => pos,
                    Err(pos) => pos,
                };

                look_right_from(key, &self.keys, pos).unwrap_or(self.keys.len())
            },
            Excluded(key) => {
                match self.keys.binary_search(key) {
                    Ok(pos) => look_left_from(key, &self.keys, pos, 1).unwrap_or(self.keys.len()),
                    Err(pos) => look_left_from(key, &self.keys, pos, 0).unwrap_or(self.keys.len()),
                }
            },
            Unbounded => {
                self.len()
            }
        };

        let skip = start.unwrap_or(self.keys.len());
        let take = if end <= skip { 0 } else { end - skip };

        let iter = Tuples { keys: &self.keys, values: &self.values, index: skip }.take(take);

        Range {
            iter
        }
    }
}

/// Iterator for an range of tuples
#[derive(Clone)]
pub struct Range<'a, K: 'a, V: 'a> {
    iter: ::std::iter::Take<Tuples<'a, K, V>>,
}

impl<'a, K: Ord + fmt::Debug, V: PartialEq + fmt::Debug> fmt::Debug for Range<'a, K, V> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self.iter.clone())
    }
}

impl<'a, K, V> Iterator for Range<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<K: Clone + Ord, V: PartialEq> Extend<(K, V)> for SortedList<K, V> {
    fn extend<T>(&mut self, iter: T) where T: IntoIterator<Item = (K, V)> {
        let mut temp = iter.into_iter().collect::<Vec<_>>();
        temp.sort_by_key(|&(ref k, _)| k.clone());

        for (k, v) in temp {
            self.insert(k, v);
        }
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
    keys: &'a Vec<K>,
    values: &'a Vec<V>,
    index: usize,
}

impl<'a, K, V> Iterator for Tuples<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.keys.len() {
            let index = self.index;
            self.index += 1;
            Some((&self.keys[index], &self.values[index]))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.keys.len() - self.index;
        (len, Some(len))
    }
}

impl<'a, K, V> Clone for Tuples<'a, K, V> {
    fn clone(&self) -> Self {
        Tuples {
            keys: self.keys.clone(),
            values: self.values.clone(),
            index: self.index,
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

    #[test]
    fn extend_worst_case() {
        use std::time::Instant;

        /// 1000, 100 => 4.08s (3.76s release) originally
        /// 1000, 100 for copy types: 0.66s (0.23s release)
        let max_key = 1000;
        let max_val = 100;
        let mut input = Vec::with_capacity(max_key * max_val);
        for key in 0..max_key {
            for val in 0..max_val {
                input.push((max_key - key, val));
            }
        }

        let began = Instant::now();

        let mut slist = SortedList::new();
        slist.extend(input);

        let elapsed = began.elapsed();
        println!("elapsed: {}.{:09}s", elapsed.as_secs(), elapsed.subsec_nanos());
    }

    #[cfg(feature = "nightly")]
    #[test]
    fn range() {
        use std::collections::Bound;
        use std::collections::Bound::*;
        use std::collections::range::RangeArgument;

        fn to_vec<'a, A: 'a + Copy, B: 'a + Copy, I: Iterator<Item=(&'a A, &'a B)>>(it: I) -> Vec<(A, B)> {
            it.map(|(a, b)| (*a, *b)).collect()
        }

        let mut list: SortedList<u32, u8> = SortedList::new();
        list.insert_only_new(1, 4);
        list.insert_only_new(0, 0);
        list.insert_only_new(0, 1);
        list.insert_only_new(2, 5);
        list.insert_only_new(0, 2);
        list.insert_only_new(3, 7);
        list.insert_only_new(0, 3);
        list.insert_only_new(2, 6);
        list.insert_only_new(4, 8);
        list.insert_only_new(6, 9);
        list.insert_only_new(6, 10);
        list.insert_only_new(9, 11);

        assert_eq!(
            to_vec(list.range((Unbounded, Included(2)))),
            vec![(0, 0), (0, 1), (0, 2), (0, 3), (1, 4), (2, 5), (2, 6)]);

        assert_eq!(
            to_vec(list.range((Unbounded, Excluded(2)))),
            vec![(0, 0), (0, 1), (0, 2), (0, 3), (1, 4)]);

        assert_eq!(
            to_vec(list.range((Included(0), Excluded(2)))),
            vec![(0, 0), (0, 1), (0, 2), (0, 3), (1, 4)]);

        assert_eq!(
            to_vec(list.range((Included(1), Excluded(2)))),
            vec![(1, 4)]);

        assert_eq!(
            to_vec(list.range((Included(2), Excluded(2)))),
            vec![]);

        assert_eq!(
            to_vec(list.range((Included(2), Included(2)))),
            vec![(2, 5), (2, 6)]);

        assert_eq!(
            to_vec(list.range((Included(2), Excluded(3)))),
            vec![(2, 5), (2, 6)]);

        assert_eq!(
            to_vec(list.range((Included(2), Included(3)))),
            vec![(2, 5), (2, 6), (3, 7)]);

        assert_eq!(
            to_vec(list.range((Included(2), Unbounded))),
            vec![(2, 5), (2, 6), (3, 7), (4, 8), (6, 9), (6, 10), (9, 11)]);

        assert_eq!(
            to_vec(list.range((Excluded(1), Unbounded))),
            vec![(2, 5), (2, 6), (3, 7), (4, 8), (6, 9), (6, 10), (9, 11)]);

        assert_eq!(
            to_vec(list.range((Excluded(0), Unbounded))),
            vec![(1, 4), (2, 5), (2, 6), (3, 7), (4, 8), (6, 9), (6, 10), (9, 11)]);

        assert_eq!(
            to_vec(list.range((Excluded(4), Unbounded))),
            vec![(6, 9), (6, 10), (9, 11)]);

        assert_eq!(
            to_vec(list.range((Included(5), Unbounded))),
            vec![(6, 9), (6, 10), (9, 11)]);

        assert_eq!(
            to_vec(list.range((Excluded(5), Unbounded))),
            vec![(6, 9), (6, 10), (9, 11)]);

        assert_eq!(
            to_vec(list.range((Excluded(6), Unbounded))),
            vec![(9, 11)]);

        assert_eq!(
            to_vec(list.range((Excluded(6), Excluded(7)))),
            vec![]);

        assert_eq!(
            to_vec(list.range((Excluded(6), Included(8)))),
            vec![]);

        assert_eq!(
            to_vec(list.range((Excluded(6), Excluded(9)))),
            vec![]);

        assert_eq!(
            to_vec(list.range((Excluded(6), Included(9)))),
            vec![(9, 11)]);

        assert_eq!(
            to_vec(list.range((Excluded(7), Included(9)))),
            vec![(9, 11)]);

        assert_eq!(
            to_vec(list.range((Included(7), Included(9)))),
            vec![(9, 11)]);

        assert_eq!(
            to_vec(list.range((Excluded(8), Included(9)))),
            vec![(9, 11)]);

        assert_eq!(
            to_vec(list.range((Included(8), Included(9)))),
            vec![(9, 11)]);

        assert_eq!(
            to_vec(list.range(..)),
            to_vec(list.iter()));
    }
}
