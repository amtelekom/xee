use xee_datasource::XmlNode;

use crate::function::XmlDS;
use crate::{atomic, error, function};

use super::item::Item;

/// An iterator over the nodes in a sequence.
pub struct NodeIter<I, N: XmlNode>
where
    I: Iterator<Item = Item<N>>,
{
    iter: I,
}

impl<I, N: XmlNode> NodeIter<I, N>
where
    I: Iterator<Item = Item<N>>,
{
    pub(crate) fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I, N: XmlNode> Iterator for NodeIter<I, N>
where
    I: Iterator<Item = Item<N>>,
{
    type Item = error::Result<N>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next();
        next.map(|v| v.to_node())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// An iterator atomizing a sequence.
pub struct AtomizedIter<'a, I, S: XmlDS>
where
    I: Iterator<Item = Item<S::Node>> + 'a,
{
    source: &'a S,
    iter: I,
    item_iter: Option<AtomizedItemIter<'a, S>>,
}

impl<'a, I, S: XmlDS> AtomizedIter<'a, I, S>
where
    I: Iterator<Item = Item<S::Node>>,
{
    pub(crate) fn new(source: &'a S, iter: I) -> AtomizedIter<'a, I, S> {
        AtomizedIter {
            source,
            iter,
            item_iter: None,
        }
    }
}

impl<'a, I, S: XmlDS> Iterator for AtomizedIter<'a, I, S>
where
    I: Iterator<Item = Item<S::Node>> + 'a,
{
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<error::Result<atomic::Atomic>> {
        loop {
            // if there there are any more atoms in this node,
            // supply those
            if let Some(item_iter) = &mut self.item_iter {
                if let Some(item) = item_iter.next() {
                    return Some(item);
                } else {
                    self.item_iter = None;
                }
            }
            // if not, move on to the next item; if there are no more items,
            // we're done
            let item = self.iter.next()?;
            self.item_iter = Some(AtomizedItemIter::new(item, self.source));
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // using iter as the lower bound is safe, as we will
        // go through each item at least once. it's harder to determine an upper
        // bound however
        let (lower, _) = self.iter.size_hint();
        (lower, None)
    }
}

/// Atomizing an individual item in a sequence.
pub enum AtomizedItemIter<'a, S: XmlDS> {
    Atomic(std::iter::Once<atomic::Atomic>),
    Node(AtomizedNodeIter),
    Array(AtomizedArrayIter<'a, S>),
    // TODO: properly handle functions; for now they error
    Erroring(std::iter::Once<error::Result<atomic::Atomic>>),
}

impl<'a, S: XmlDS> AtomizedItemIter<'a, S> {
    pub(crate) fn new(item: Item<S::Node>, source: &'a S) -> Self {
        match item {
            Item::Atomic(a) => Self::Atomic(std::iter::once(a)),
            Item::Node(n) => Self::Node(AtomizedNodeIter::new(n, source)),
            Item::Function(function) => match function {
                function::Function::Array(a) => Self::Array(AtomizedArrayIter::new(a, source)),
                _ => Self::Erroring(std::iter::once(Err(error::Error::FOTY0013))),
            },
        }
    }
}

impl<S: XmlDS> Iterator for AtomizedItemIter<'_, S> {
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Atomic(iter) => iter.next().map(Ok),
            Self::Node(iter) => iter.next().map(Ok),
            Self::Array(iter) => iter.next(),
            Self::Erroring(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Atomic(iter) => iter.size_hint(),
            Self::Node(iter) => iter.size_hint(),
            Self::Array(iter) => iter.size_hint(),
            Self::Erroring(iter) => iter.size_hint(),
        }
    }
}

/// Atomizing a node
pub struct AtomizedNodeIter {
    typed_value: Vec<atomic::Atomic>,
    typed_value_index: usize,
}

impl AtomizedNodeIter {
    fn new<S: XmlDS>(node: S::Node, source: &S) -> Self {
        let s = source.string_value(node);
        let typed_value = vec![atomic::Atomic::Untyped(s.into())];
        Self {
            typed_value,
            typed_value_index: 0,
        }
    }
}

impl Iterator for AtomizedNodeIter {
    type Item = atomic::Atomic;

    fn next(&mut self) -> Option<Self::Item> {
        if self.typed_value_index < self.typed_value.len() {
            let item = self.typed_value[self.typed_value_index].clone();
            self.typed_value_index += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.typed_value.len() - self.typed_value_index;
        (remaining, Some(remaining))
    }
}

/// Atomizing a XPath array
pub struct AtomizedArrayIter<'a, S: XmlDS> {
    source: &'a S,
    array: function::Array<S::Node>,
    array_index: usize,
    iter: Option<std::vec::IntoIter<error::Result<atomic::Atomic>>>,
}

impl<'a, S: XmlDS> AtomizedArrayIter<'a, S> {
    fn new(array: function::Array<S::Node>, source: &'a S) -> Self {
        Self {
            source,
            array,
            array_index: 0,
            iter: None,
        }
    }
}

impl<S: XmlDS> Iterator for AtomizedArrayIter<'_, S> {
    type Item = error::Result<atomic::Atomic>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // if there there are any more atoms in this array entry,
            // supply those
            if let Some(iter) = &mut self.iter {
                if let Some(item) = iter.next() {
                    return Some(item);
                } else {
                    self.iter = None;
                }
            }
            let array = &self.array.0;
            // if we're at the end of the array, we're done
            if self.array_index >= array.len() {
                return None;
            }
            let sequence = &array[self.array_index];
            self.array_index += 1;
            // TODO: we have lifetime whackamole issues here, because
            // an array reference cannot live long enough, but an owned
            // array also cannot live long enough. So we collect things
            // into a vector...
            let v = sequence.atomized(self.source).collect::<Vec<_>>();
            self.iter = Some(v.into_iter());
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // we will have at least as many entries as in the array, but
        // we don't really know the upper bound
        let remaining = self.array.0.len() - self.array_index;
        (remaining, None)
    }
}

pub(crate) fn one<'a, T>(mut iter: impl Iterator<Item = T> + 'a) -> error::Result<T> {
    if let Some(one) = iter.next() {
        if iter.next().is_none() {
            Ok(one)
        } else {
            Err(error::Error::XPTY0004(
                "iter::one called with more than one item",
            ))
        }
    } else {
        Err(error::Error::XPTY0004(
            "iter::one called with an empty iterator",
        ))
    }
}

pub(crate) fn option<'a, T>(mut iter: impl Iterator<Item = T> + 'a) -> error::Result<Option<T>> {
    if let Some(one) = iter.next() {
        if iter.next().is_none() {
            Ok(Some(one))
        } else {
            Err(error::Error::XPTY0004(
                "iter::option called with more than one item",
            ))
        }
    } else {
        Ok(None)
    }
}
