// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// This is pretty much entirely stolen from TreeSet, since BTreeMap has an identical interface
// to TreeMap

use core::cmp::Ordering::{self, Less, Greater, Equal};
use core::cmp::{min, max};
use core::fmt::Debug;
use core::fmt;
use core::iter::{Peekable, FromIterator, FusedIterator};
use core::ops::{BitOr, BitAnd, BitXor, Sub};

use core::borrow::Borrow;
use super::map::{BTreeMap, Keys};
use super::Recover;
use super::set::Iter as SetIter;
use super::map::Keys as MapKeys;
use super::map::Range as MapRange;
use super::map::Iter as MapIter;
use super::map::last_leaf_edge;

// FIXME(conventions): implement bounded iterators

/// A set based on a B-Tree.
///
/// See [`BTreeMap`]'s documentation for a detailed discussion of this collection's performance
/// benefits and drawbacks.
///
/// It is a logic error for an item to be modified in such a way that the item's ordering relative
/// to any other item, as determined by the [`Ord`] trait, changes while it is in the set. This is
/// normally only possible through [`Cell`], [`RefCell`], global state, I/O, or unsafe code.
///
/// [`BTreeMap`]: struct.BTreeMap.html
/// [`Ord`]: ../../std/cmp/trait.Ord.html
/// [`Cell`]: ../../std/cell/struct.Cell.html
/// [`RefCell`]: ../../std/cell/struct.RefCell.html
///
/// # Examples
///
/// ```
/// use std::collections::BTreeSet;
///
/// // Type inference lets us omit an explicit type signature (which
/// // would be `BTreeSet<&str>` in this example).
/// let mut books = BTreeSet::new();
///
/// // Add some books.
/// books.insert("A Dance With Dragons");
/// books.insert("To Kill a Mockingbird");
/// books.insert("The Odyssey");
/// books.insert("The Great Gatsby");
///
/// // Check for a specific one.
/// if !books.contains("The Winds of Winter") {
///     println!("We have {} books, but The Winds of Winter ain't one.",
///              books.len());
/// }
///
/// // Remove a book.
/// books.remove("The Odyssey");
///
/// // Iterate over everything.
/// for book in &books {
///     println!("{}", book);
/// }
/// ```
#[derive(Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct BTreeSet<T> {
    pub(crate) map: BTreeMap<T, ()>,
}

/// An iterator over the items of a `BTreeSet`.
///
/// This `struct` is created by the [`iter`] method on [`BTreeSet`].
/// See its documentation for more.
///
/// [`BTreeSet`]: struct.BTreeSet.html
/// [`iter`]: struct.BTreeSet.html#method.iter

pub struct Iter<'a, T: 'a> {
    iter: Keys<'a, T, ()>,
}


impl<'a, T: 'a + fmt::Debug> fmt::Debug for Iter<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Iter")
         .field(&self.iter.clone())
         .finish()
    }
}

/// An owning iterator over the items of a `BTreeSet`.
///
/// This `struct` is created by the [`into_iter`] method on [`BTreeSet`][`BTreeSet`]
/// (provided by the `IntoIterator` trait). See its documentation for more.
///
/// [`BTreeSet`]: struct.BTreeSet.html
/// [`into_iter`]: struct.BTreeSet.html#method.into_iter

#[derive(Debug)]
pub struct IntoIter<T> {
    iter: super::map::IntoIter<T, ()>,
}

/// An iterator over a sub-range of items in a `BTreeSet`.
///
/// This `struct` is created by the [`range`] method on [`BTreeSet`].
/// See its documentation for more.
///
/// [`BTreeSet`]: struct.BTreeSet.html
/// [`range`]: struct.BTreeSet.html#method.range
#[derive(Debug)]

pub struct Range<'a, T: 'a> {
    iter: super::map::Range<'a, T, ()>,
}

/// A lazy iterator producing elements in the difference of `BTreeSet`s.
///
/// This `struct` is created by the [`difference`] method on [`BTreeSet`].
/// See its documentation for more.
///
/// [`BTreeSet`]: struct.BTreeSet.html
/// [`difference`]: struct.BTreeSet.html#method.difference

pub struct Difference<'a, T: 'a> {
    a: Peekable<Iter<'a, T>>,
    b: Peekable<Iter<'a, T>>,
}


impl<'a, T: 'a + fmt::Debug> fmt::Debug for Difference<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Difference")
         .field(&self.a)
         .field(&self.b)
         .finish()
    }
}

/// A lazy iterator producing elements in the symmetric difference of `BTreeSet`s.
///
/// This `struct` is created by the [`symmetric_difference`] method on
/// [`BTreeSet`]. See its documentation for more.
///
/// [`BTreeSet`]: struct.BTreeSet.html
/// [`symmetric_difference`]: struct.BTreeSet.html#method.symmetric_difference

pub struct SymmetricDifference<'a, T: 'a> {
    a: Peekable<Iter<'a, T>>,
    b: Peekable<Iter<'a, T>>,
}


impl<'a, T: 'a + fmt::Debug> fmt::Debug for SymmetricDifference<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("SymmetricDifference")
         .field(&self.a)
         .field(&self.b)
         .finish()
    }
}

/// A lazy iterator producing elements in the intersection of `BTreeSet`s.
///
/// This `struct` is created by the [`intersection`] method on [`BTreeSet`].
/// See its documentation for more.
///
/// [`BTreeSet`]: struct.BTreeSet.html
/// [`intersection`]: struct.BTreeSet.html#method.intersection

pub struct Intersection<'a, T: 'a> {
    a: Peekable<Iter<'a, T>>,
    b: Peekable<Iter<'a, T>>,
}


impl<'a, T: 'a + fmt::Debug> fmt::Debug for Intersection<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Intersection")
         .field(&self.a)
         .field(&self.b)
         .finish()
    }
}

/// A lazy iterator producing elements in the union of `BTreeSet`s.
///
/// This `struct` is created by the [`union`] method on [`BTreeSet`].
/// See its documentation for more.
///
/// [`BTreeSet`]: struct.BTreeSet.html
/// [`union`]: struct.BTreeSet.html#method.union

pub struct Union<'a, T: 'a> {
    a: Peekable<Iter<'a, T>>,
    b: Peekable<Iter<'a, T>>,
}


impl<'a, T: 'a + fmt::Debug> fmt::Debug for Union<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Union")
         .field(&self.a)
         .field(&self.b)
         .finish()
    }
}

impl<T: Ord> BTreeSet<T> {
    /// Makes a new `BTreeSet` with a reasonable choice of B.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_mut)]
    /// use std::collections::BTreeSet;
    ///
    /// let mut set: BTreeSet<i32> = BTreeSet::new();
    /// ```

    pub fn new() -> BTreeSet<T> {
        BTreeSet { map: BTreeMap::new() }
    }
}

impl<T> BTreeSet<T> {
    /// Gets an iterator that visits the values in the `BTreeSet` in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let set: BTreeSet<usize> = [1, 2, 3].iter().cloned().collect();
    /// let mut set_iter = set.iter();
    /// assert_eq!(set_iter.next(), Some(&1));
    /// assert_eq!(set_iter.next(), Some(&2));
    /// assert_eq!(set_iter.next(), Some(&3));
    /// assert_eq!(set_iter.next(), None);
    /// ```
    ///
    /// Values returned by the iterator are returned in ascending order:
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let set: BTreeSet<usize> = [3, 1, 2].iter().cloned().collect();
    /// let mut set_iter = set.iter();
    /// assert_eq!(set_iter.next(), Some(&1));
    /// assert_eq!(set_iter.next(), Some(&2));
    /// assert_eq!(set_iter.next(), Some(&3));
    /// assert_eq!(set_iter.next(), None);
    /// ```

    pub fn iter(&self) -> Iter<T> {
        Iter { iter: self.map.keys() }
    }
}

impl<T: Ord> BTreeSet<T> {
    /// Visits the values representing the difference,
    /// i.e. the values that are in `self` but not in `other`,
    /// in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let mut a = BTreeSet::new();
    /// a.insert(1);
    /// a.insert(2);
    ///
    /// let mut b = BTreeSet::new();
    /// b.insert(2);
    /// b.insert(3);
    ///
    /// let diff: Vec<_> = a.difference(&b).cloned().collect();
    /// assert_eq!(diff, [1]);
    /// ```

    pub fn difference<'a>(&'a self, other: &'a BTreeSet<T>) -> Difference<'a, T> {
        Difference {
            a: self.iter().peekable(),
            b: other.iter().peekable(),
        }
    }

    /// Visits the values representing the symmetric difference,
    /// i.e. the values that are in `self` or in `other` but not in both,
    /// in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let mut a = BTreeSet::new();
    /// a.insert(1);
    /// a.insert(2);
    ///
    /// let mut b = BTreeSet::new();
    /// b.insert(2);
    /// b.insert(3);
    ///
    /// let sym_diff: Vec<_> = a.symmetric_difference(&b).cloned().collect();
    /// assert_eq!(sym_diff, [1, 3]);
    /// ```

    pub fn symmetric_difference<'a>(&'a self,
                                    other: &'a BTreeSet<T>)
                                    -> SymmetricDifference<'a, T> {
        SymmetricDifference {
            a: self.iter().peekable(),
            b: other.iter().peekable(),
        }
    }

    /// Visits the values representing the intersection,
    /// i.e. the values that are both in `self` and `other`,
    /// in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let mut a = BTreeSet::new();
    /// a.insert(1);
    /// a.insert(2);
    ///
    /// let mut b = BTreeSet::new();
    /// b.insert(2);
    /// b.insert(3);
    ///
    /// let intersection: Vec<_> = a.intersection(&b).cloned().collect();
    /// assert_eq!(intersection, [2]);
    /// ```

    pub fn intersection<'a>(&'a self, other: &'a BTreeSet<T>) -> Intersection<'a, T> {
        Intersection {
            a: self.iter().peekable(),
            b: other.iter().peekable(),
        }
    }

    /// Visits the values representing the union,
    /// i.e. all the values in `self` or `other`, without duplicates,
    /// in ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let mut a = BTreeSet::new();
    /// a.insert(1);
    ///
    /// let mut b = BTreeSet::new();
    /// b.insert(2);
    ///
    /// let union: Vec<_> = a.union(&b).cloned().collect();
    /// assert_eq!(union, [1, 2]);
    /// ```

    pub fn union<'a>(&'a self, other: &'a BTreeSet<T>) -> Union<'a, T> {
        Union {
            a: self.iter().peekable(),
            b: other.iter().peekable(),
        }
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let mut v = BTreeSet::new();
    /// assert_eq!(v.len(), 0);
    /// v.insert(1);
    /// assert_eq!(v.len(), 1);
    /// ```

    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let mut v = BTreeSet::new();
    /// assert!(v.is_empty());
    /// v.insert(1);
    /// assert!(!v.is_empty());
    /// ```

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears the set, removing all values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let mut v = BTreeSet::new();
    /// v.insert(1);
    /// v.clear();
    /// assert!(v.is_empty());
    /// ```

    pub fn clear(&mut self) {
        self.map.clear()
    }

    /// Returns `true` if the set contains a value.
    ///
    /// The value may be any borrowed form of the set's value type,
    /// but the ordering on the borrowed form *must* match the
    /// ordering on the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let set: BTreeSet<_> = [1, 2, 3].iter().cloned().collect();
    /// assert_eq!(set.contains(&1), true);
    /// assert_eq!(set.contains(&4), false);
    /// ```

    pub fn contains<Q: ?Sized>(&self, value: &Q) -> bool
        where T: Borrow<Q>,
              Q: Ord
    {
        self.map.contains_key(value)
    }

    /// Returns a reference to the value in the set, if any, that is equal to the given value.
    ///
    /// The value may be any borrowed form of the set's value type,
    /// but the ordering on the borrowed form *must* match the
    /// ordering on the value type.

    pub fn get<Q: ?Sized>(&self, value: &Q) -> Option<&T>
        where T: Borrow<Q>,
              Q: Ord
    {
        Recover::get(&self.map, value)
    }

    /// Returns `true` if `self` has no elements in common with `other`.
    /// This is equivalent to checking for an empty intersection.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let a: BTreeSet<_> = [1, 2, 3].iter().cloned().collect();
    /// let mut b = BTreeSet::new();
    ///
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(4);
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(1);
    /// assert_eq!(a.is_disjoint(&b), false);
    /// ```

    pub fn is_disjoint(&self, other: &BTreeSet<T>) -> bool {
        self.intersection(other).next().is_none()
    }

    /// Returns `true` if the set is a subset of another,
    /// i.e. `other` contains at least all the values in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let sup: BTreeSet<_> = [1, 2, 3].iter().cloned().collect();
    /// let mut set = BTreeSet::new();
    ///
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(2);
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(4);
    /// assert_eq!(set.is_subset(&sup), false);
    /// ```

    pub fn is_subset(&self, other: &BTreeSet<T>) -> bool {
        // Stolen from TreeMap
        let mut x = self.iter();
        let mut y = other.iter();
        let mut a = x.next();
        let mut b = y.next();
        while a.is_some() {
            if b.is_none() {
                return false;
            }

            let a1 = a.unwrap();
            let b1 = b.unwrap();

            match b1.cmp(a1) {
                Less => (),
                Greater => return false,
                Equal => a = x.next(),
            }

            b = y.next();
        }
        true
    }

    /// Returns `true` if the set is a superset of another,
    /// i.e. `self` contains at least all the values in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let sub: BTreeSet<_> = [1, 2].iter().cloned().collect();
    /// let mut set = BTreeSet::new();
    ///
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(0);
    /// set.insert(1);
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(2);
    /// assert_eq!(set.is_superset(&sub), true);
    /// ```

    pub fn is_superset(&self, other: &BTreeSet<T>) -> bool {
        other.is_subset(self)
    }

    /// Adds a value to the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    ///
    /// If the set did have this value present, `false` is returned, and the
    /// entry is not updated. See the [module-level documentation] for more.
    ///
    /// [module-level documentation]: index.html#insert-and-complex-keys
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let mut set = BTreeSet::new();
    ///
    /// assert_eq!(set.insert(2), true);
    /// assert_eq!(set.insert(2), false);
    /// assert_eq!(set.len(), 1);
    /// ```

    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ()).is_none()
    }

    /// Inserts the value into the set, returns the index into the keys
    /// of the newly inserted value
    ///
    /// If you want the key, get it with `self.map.keys_mut(idx)`
    ///
    /// Of course, if you remove something afterwards, the index will be invalid.
    pub fn insert_return_index(&mut self, value: T) -> SetIter<T> {

        use super::node::InsertResult::*;
        use super::search::SearchResult::*;

        let main_handle;

        match super::search::search_tree(self.map.root.as_mut(), &value) {
            Found(handle) => {
                // stripped from the source of OccupiedEntry::insert()
                main_handle = handle;
            }
            GoDown(handle) => {
                let mut self_length = &mut self.map.length;

                // stripped from the source of VacantEntry::insert()

                *self_length += 1;

                // let out_ptr;
                let mut ins_k;
                let mut ins_v;
                let mut ins_edge;

                let mut cur_parent = match handle.insert(value, ()) {
                    (Fit(handle), _) => { main_handle = handle; },
                    (Split(left, k, v, right), ptr) => {
                        ins_k = k;
                        ins_v = v;
                        ins_edge = right;
                        // out_ptr = ptr;
                        left.ascend().map_err(|n| n.into_root_mut())
                    }
                };

                loop {
                    match cur_parent {
                        Ok(parent) => {
                            match parent.insert(ins_k, ins_v, ins_edge) {
                                Fit(handle) => { main_handle = handle; break; },
                                Split(left, k, v, right) => {
                                    ins_k = k;
                                    ins_v = v;
                                    ins_edge = right;
                                    cur_parent = left.ascend().map_err(|n| n.into_root_mut());
                                }
                            }
                        }
                        Err(root) => {
                            let mut lev = root.push_level();
                            let len = lev.len();
                            lev.push(ins_k, ins_v, ins_edge);
                            main_handle = lev;
                            break;
                        }
                    }
                }
            }
        }
/*
        pub struct Iter<'a, T: 'a> {
            iter: Keys<'a, T, ()>,
        }

        pub struct Keys<'a, K: 'a, V: 'a> {
            inner: Iter<'a, K, V>,
        }

        pub struct Iter<'a, K: 'a, V: 'a> {
            pub(in custom_btreeset) range: Range<'a, K, V>,
            pub(in custom_btreeset) length: usize,
        }
*/
        SetIter {
            iter: MapKeys {
                inner: MapIter {
                    range: MapRange {
                        front: main_handle,
                        back: last_leaf_edge(self.map.root.as_ref()),
                    },
                    length: self.map.length,
                }
            }
        }
    }

    /// Adds a value to the set, replacing the existing value, if any, that is equal to the given
    /// one. Returns the replaced value.

    pub fn replace(&mut self, value: T) -> Option<T> {
        Recover::replace(&mut self.map, value)
    }

    /// Removes a value from the set. Returns `true` if the value was
    /// present in the set.
    ///
    /// The value may be any borrowed form of the set's value type,
    /// but the ordering on the borrowed form *must* match the
    /// ordering on the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let mut set = BTreeSet::new();
    ///
    /// set.insert(2);
    /// assert_eq!(set.remove(&2), true);
    /// assert_eq!(set.remove(&2), false);
    /// ```

    pub fn remove<Q: ?Sized>(&mut self, value: &Q) -> bool
        where T: Borrow<Q>,
              Q: Ord
    {
        self.map.remove(value).is_some()
    }

    /// Removes and returns the value in the set, if any, that is equal to the given one.
    ///
    /// The value may be any borrowed form of the set's value type,
    /// but the ordering on the borrowed form *must* match the
    /// ordering on the value type.

    pub fn take<Q: ?Sized>(&mut self, value: &Q) -> Option<T>
        where T: Borrow<Q>,
              Q: Ord
    {
        Recover::take(&mut self.map, value)
    }

    /// Moves all elements from `other` into `Self`, leaving `other` empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let mut a = BTreeSet::new();
    /// a.insert(1);
    /// a.insert(2);
    /// a.insert(3);
    ///
    /// let mut b = BTreeSet::new();
    /// b.insert(3);
    /// b.insert(4);
    /// b.insert(5);
    ///
    /// a.append(&mut b);
    ///
    /// assert_eq!(a.len(), 5);
    /// assert_eq!(b.len(), 0);
    ///
    /// assert!(a.contains(&1));
    /// assert!(a.contains(&2));
    /// assert!(a.contains(&3));
    /// assert!(a.contains(&4));
    /// assert!(a.contains(&5));
    /// ```

    pub fn append(&mut self, other: &mut Self) {
        self.map.append(&mut other.map);
    }

    /// Splits the collection into two at the given key. Returns everything after the given key,
    /// including the key.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut a = BTreeMap::new();
    /// a.insert(1, "a");
    /// a.insert(2, "b");
    /// a.insert(3, "c");
    /// a.insert(17, "d");
    /// a.insert(41, "e");
    ///
    /// let b = a.split_off(&3);
    ///
    /// assert_eq!(a.len(), 2);
    /// assert_eq!(b.len(), 3);
    ///
    /// assert_eq!(a[&1], "a");
    /// assert_eq!(a[&2], "b");
    ///
    /// assert_eq!(b[&3], "c");
    /// assert_eq!(b[&17], "d");
    /// assert_eq!(b[&41], "e");
    /// ```

    pub fn split_off<Q: ?Sized + Ord>(&mut self, key: &Q) -> Self where T: Borrow<Q> {
        BTreeSet { map: self.map.split_off(key) }
    }
}


impl<T: Ord> FromIterator<T> for BTreeSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> BTreeSet<T> {
        let mut set = BTreeSet::new();
        set.extend(iter);
        set
    }
}


impl<T> IntoIterator for BTreeSet<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    /// Gets an iterator for moving out the `BTreeSet`'s contents.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let set: BTreeSet<usize> = [1, 2, 3, 4].iter().cloned().collect();
    ///
    /// let v: Vec<_> = set.into_iter().collect();
    /// assert_eq!(v, [1, 2, 3, 4]);
    /// ```
    fn into_iter(self) -> IntoIter<T> {
        IntoIter { iter: self.map.into_iter() }
    }
}


impl<'a, T> IntoIterator for &'a BTreeSet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}


impl<T: Ord> Extend<T> for BTreeSet<T> {
    #[inline]
    fn extend<Iter: IntoIterator<Item = T>>(&mut self, iter: Iter) {
        for elem in iter {
            self.insert(elem);
        }
    }
}


impl<'a, T: 'a + Ord + Copy> Extend<&'a T> for BTreeSet<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}


impl<T: Ord> Default for BTreeSet<T> {
    /// Makes an empty `BTreeSet<T>` with a reasonable choice of B.
    fn default() -> BTreeSet<T> {
        BTreeSet::new()
    }
}


impl<'a, 'b, T: Ord + Clone> Sub<&'b BTreeSet<T>> for &'a BTreeSet<T> {
    type Output = BTreeSet<T>;

    /// Returns the difference of `self` and `rhs` as a new `BTreeSet<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let a: BTreeSet<_> = vec![1, 2, 3].into_iter().collect();
    /// let b: BTreeSet<_> = vec![3, 4, 5].into_iter().collect();
    ///
    /// let result = &a - &b;
    /// let result_vec: Vec<_> = result.into_iter().collect();
    /// assert_eq!(result_vec, [1, 2]);
    /// ```
    fn sub(self, rhs: &BTreeSet<T>) -> BTreeSet<T> {
        self.difference(rhs).cloned().collect()
    }
}


impl<'a, 'b, T: Ord + Clone> BitXor<&'b BTreeSet<T>> for &'a BTreeSet<T> {
    type Output = BTreeSet<T>;

    /// Returns the symmetric difference of `self` and `rhs` as a new `BTreeSet<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let a: BTreeSet<_> = vec![1, 2, 3].into_iter().collect();
    /// let b: BTreeSet<_> = vec![2, 3, 4].into_iter().collect();
    ///
    /// let result = &a ^ &b;
    /// let result_vec: Vec<_> = result.into_iter().collect();
    /// assert_eq!(result_vec, [1, 4]);
    /// ```
    fn bitxor(self, rhs: &BTreeSet<T>) -> BTreeSet<T> {
        self.symmetric_difference(rhs).cloned().collect()
    }
}


impl<'a, 'b, T: Ord + Clone> BitAnd<&'b BTreeSet<T>> for &'a BTreeSet<T> {
    type Output = BTreeSet<T>;

    /// Returns the intersection of `self` and `rhs` as a new `BTreeSet<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let a: BTreeSet<_> = vec![1, 2, 3].into_iter().collect();
    /// let b: BTreeSet<_> = vec![2, 3, 4].into_iter().collect();
    ///
    /// let result = &a & &b;
    /// let result_vec: Vec<_> = result.into_iter().collect();
    /// assert_eq!(result_vec, [2, 3]);
    /// ```
    fn bitand(self, rhs: &BTreeSet<T>) -> BTreeSet<T> {
        self.intersection(rhs).cloned().collect()
    }
}


impl<'a, 'b, T: Ord + Clone> BitOr<&'b BTreeSet<T>> for &'a BTreeSet<T> {
    type Output = BTreeSet<T>;

    /// Returns the union of `self` and `rhs` as a new `BTreeSet<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// let a: BTreeSet<_> = vec![1, 2, 3].into_iter().collect();
    /// let b: BTreeSet<_> = vec![3, 4, 5].into_iter().collect();
    ///
    /// let result = &a | &b;
    /// let result_vec: Vec<_> = result.into_iter().collect();
    /// assert_eq!(result_vec, [1, 2, 3, 4, 5]);
    /// ```
    fn bitor(self, rhs: &BTreeSet<T>) -> BTreeSet<T> {
        self.union(rhs).cloned().collect()
    }
}


impl<T: Debug> Debug for BTreeSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}


impl<'a, T> Clone for Iter<'a, T> {
    fn clone(&self) -> Iter<'a, T> {
        Iter { iter: self.iter.clone() }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        self.iter.next_back()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize { self.iter.len() }
}


impl<'a, T> FusedIterator for Iter<'a, T> {}


impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.iter.next().map(|(k, _)| k)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back().map(|(k, _)| k)
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize { self.iter.len() }
}


impl<T> FusedIterator for IntoIter<T> {}


impl<'a, T> Clone for Range<'a, T> {
    fn clone(&self) -> Range<'a, T> {
        Range { iter: self.iter.clone() }
    }
}


impl<'a, T> Iterator for Range<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.iter.next().map(|(k, _)| k)
    }
}


impl<'a, T> DoubleEndedIterator for Range<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        self.iter.next_back().map(|(k, _)| k)
    }
}


impl<'a, T> FusedIterator for Range<'a, T> {}

/// Compare `x` and `y`, but return `short` if x is None and `long` if y is None
fn cmp_opt<T: Ord>(x: Option<&T>, y: Option<&T>, short: Ordering, long: Ordering) -> Ordering {
    match (x, y) {
        (None, _) => short,
        (_, None) => long,
        (Some(x1), Some(y1)) => x1.cmp(y1),
    }
}


impl<'a, T> Clone for Difference<'a, T> {
    fn clone(&self) -> Difference<'a, T> {
        Difference {
            a: self.a.clone(),
            b: self.b.clone(),
        }
    }
}

impl<'a, T: Ord> Iterator for Difference<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        loop {
            match cmp_opt(self.a.peek(), self.b.peek(), Less, Less) {
                Less => return self.a.next(),
                Equal => {
                    self.a.next();
                    self.b.next();
                }
                Greater => {
                    self.b.next();
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let a_len = self.a.len();
        let b_len = self.b.len();
        (a_len.saturating_sub(b_len), Some(a_len))
    }
}


impl<'a, T: Ord> FusedIterator for Difference<'a, T> {}


impl<'a, T> Clone for SymmetricDifference<'a, T> {
    fn clone(&self) -> SymmetricDifference<'a, T> {
        SymmetricDifference {
            a: self.a.clone(),
            b: self.b.clone(),
        }
    }
}

impl<'a, T: Ord> Iterator for SymmetricDifference<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        loop {
            match cmp_opt(self.a.peek(), self.b.peek(), Greater, Less) {
                Less => return self.a.next(),
                Equal => {
                    self.a.next();
                    self.b.next();
                }
                Greater => return self.b.next(),
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.a.len() + self.b.len()))
    }
}


impl<'a, T: Ord> FusedIterator for SymmetricDifference<'a, T> {}


impl<'a, T> Clone for Intersection<'a, T> {
    fn clone(&self) -> Intersection<'a, T> {
        Intersection {
            a: self.a.clone(),
            b: self.b.clone(),
        }
    }
}

impl<'a, T: Ord> Iterator for Intersection<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        loop {
            match Ord::cmp(self.a.peek()?, self.b.peek()?) {
                Less => {
                    self.a.next();
                }
                Equal => {
                    self.b.next();
                    return self.a.next();
                }
                Greater => {
                    self.b.next();
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(min(self.a.len(), self.b.len())))
    }
}


impl<'a, T: Ord> FusedIterator for Intersection<'a, T> {}


impl<'a, T> Clone for Union<'a, T> {
    fn clone(&self) -> Union<'a, T> {
        Union {
            a: self.a.clone(),
            b: self.b.clone(),
        }
    }
}

impl<'a, T: Ord> Iterator for Union<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        match cmp_opt(self.a.peek(), self.b.peek(), Greater, Less) {
            Less => self.a.next(),
            Equal => {
                self.b.next();
                self.a.next()
            }
            Greater => self.b.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let a_len = self.a.len();
        let b_len = self.b.len();
        (max(a_len, b_len), Some(a_len + b_len))
    }
}


impl<'a, T: Ord> FusedIterator for Union<'a, T> {}
